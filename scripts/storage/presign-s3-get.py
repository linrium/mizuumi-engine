#!/usr/bin/env python3
"""Create an AWS Signature V4 presigned GET URL using only the standard library."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import hmac
import os
from urllib.parse import quote, urlencode, urlsplit, urlunsplit


def hmac_sha256(key: bytes, value: str) -> bytes:
    return hmac.new(key, value.encode("utf-8"), hashlib.sha256).digest()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--endpoint", required=True)
    parser.add_argument("--bucket", required=True)
    parser.add_argument("--key", required=True)
    parser.add_argument("--region", default="us-east-1")
    parser.add_argument("--expires", type=int, default=3600)
    args = parser.parse_args()

    if not 1 <= args.expires <= 604800:
        parser.error("--expires must be between 1 and 604800 seconds")

    access_key = os.environ.get("AWS_ACCESS_KEY_ID", "")
    secret_key = os.environ.get("AWS_SECRET_ACCESS_KEY", "")
    session_token = os.environ.get("AWS_SESSION_TOKEN", "")
    if not access_key or not secret_key:
        parser.error("AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY must be set")

    endpoint = urlsplit(args.endpoint)
    if endpoint.scheme != "https" or not endpoint.hostname:
        parser.error("--endpoint must be an https URL")
    if endpoint.path not in ("", "/") or endpoint.query or endpoint.fragment:
        parser.error("--endpoint must not contain a path, query, or fragment")

    now = dt.datetime.now(dt.timezone.utc)
    amz_date = now.strftime("%Y%m%dT%H%M%SZ")
    date_stamp = now.strftime("%Y%m%d")
    service = "s3"
    scope = f"{date_stamp}/{args.region}/{service}/aws4_request"
    canonical_uri = f"/{quote(args.bucket, safe='-_.~')}/{quote(args.key, safe='/-_.~')}"
    host = endpoint.netloc

    query = {
        "X-Amz-Algorithm": "AWS4-HMAC-SHA256",
        "X-Amz-Credential": f"{access_key}/{scope}",
        "X-Amz-Date": amz_date,
        "X-Amz-Expires": str(args.expires),
        "X-Amz-SignedHeaders": "host",
    }
    if session_token:
        query["X-Amz-Security-Token"] = session_token
    canonical_query = urlencode(
        sorted(query.items()), quote_via=quote, safe="-_.~"
    )
    canonical_request = "\n".join(
        (
            "GET",
            canonical_uri,
            canonical_query,
            f"host:{host}\n",
            "host",
            "UNSIGNED-PAYLOAD",
        )
    )
    string_to_sign = "\n".join(
        (
            "AWS4-HMAC-SHA256",
            amz_date,
            scope,
            hashlib.sha256(canonical_request.encode("utf-8")).hexdigest(),
        )
    )
    signing_key = hmac_sha256(
        hmac_sha256(
            hmac_sha256(
                hmac_sha256(f"AWS4{secret_key}".encode("utf-8"), date_stamp),
                args.region,
            ),
            service,
        ),
        "aws4_request",
    )
    signature = hmac.new(
        signing_key, string_to_sign.encode("utf-8"), hashlib.sha256
    ).hexdigest()
    final_query = f"{canonical_query}&X-Amz-Signature={signature}"
    print(urlunsplit((endpoint.scheme, endpoint.netloc, canonical_uri, final_query, "")))


if __name__ == "__main__":
    main()
