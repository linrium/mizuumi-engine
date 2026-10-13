from __future__ import annotations

import json
import os
import unittest
from unittest.mock import patch

import duckdb

from server import configure_unity_catalogs, configured_unity_catalogs, sql_identifier


class RecordingConnection:
    def __init__(self) -> None:
        self.statements: list[str] = []

    def execute(self, statement: str) -> None:
        self.statements.append(statement)


class ConfiguredUnityCatalogsTest(unittest.TestCase):
    def test_reads_multiple_catalogs(self) -> None:
        catalogs = [
            {"name": "unity", "alias": "unity", "defaultSchema": "demo"},
            {"name": "payments", "alias": "payments", "defaultSchema": "bronze"},
            {"name": "vietjet", "alias": "vietjet", "defaultSchema": "bronze"},
            {"name": "hdbank", "alias": "hdbank", "defaultSchema": "bronze"},
            {"name": "vikki", "alias": "vikki", "defaultSchema": "bronze"},
            {"name": "crm", "alias": "crm", "defaultSchema": "bronze"},
        ]
        with patch.dict(os.environ, {"UNITY_CATALOGS": json.dumps(catalogs)}, clear=True):
            self.assertEqual(
                configured_unity_catalogs(),
                [
                    ("unity", "unity", "demo"),
                    ("payments", "payments", "bronze"),
                    ("vietjet", "vietjet", "bronze"),
                    ("hdbank", "hdbank", "bronze"),
                    ("vikki", "vikki", "bronze"),
                    ("crm", "crm", "bronze"),
                ],
            )

    def test_alias_defaults_to_catalog_name(self) -> None:
        catalogs = [{"name": "payments", "defaultSchema": "bronze"}]
        with patch.dict(os.environ, {"UNITY_CATALOGS": json.dumps(catalogs)}, clear=True):
            self.assertEqual(
                configured_unity_catalogs(), [("payments", "payments", "bronze")]
            )

    def test_rejects_duplicate_aliases(self) -> None:
        catalogs = [
            {"name": "unity", "alias": "data", "defaultSchema": "demo"},
            {"name": "payments", "alias": "data", "defaultSchema": "bronze"},
        ]
        with patch.dict(os.environ, {"UNITY_CATALOGS": json.dumps(catalogs)}, clear=True):
            with self.assertRaisesRegex(RuntimeError, "duplicate Unity Catalog alias"):
                configured_unity_catalogs()

    def test_uses_legacy_single_catalog_configuration(self) -> None:
        with patch.dict(
            os.environ,
            {
                "UNITY_CATALOG_NAME": "legacy",
                "UNITY_CATALOG_DEFAULT_SCHEMA": "default_schema",
            },
            clear=True,
        ):
            self.assertEqual(
                configured_unity_catalogs(),
                [("legacy", "unity", "default_schema")],
            )


class SqlIdentifierTest(unittest.TestCase):
    def test_quotes_identifier(self) -> None:
        self.assertEqual(sql_identifier('pay"ments'), '"pay""ments"')


class TimestampResultTest(unittest.TestCase):
    def test_fetches_timezone_aware_timestamp(self) -> None:
        connection = duckdb.connect()
        try:
            row = connection.execute(
                "SELECT TIMESTAMPTZ '2026-01-01 00:00:00+07:00'"
            ).fetchone()
        finally:
            connection.close()

        self.assertIsNotNone(row)
        assert row is not None
        self.assertIsNotNone(row[0].tzinfo)


class ConfigureUnityCatalogsTest(unittest.TestCase):
    def test_attaches_every_configured_catalog(self) -> None:
        catalogs = [
            {"name": "unity", "alias": "unity", "defaultSchema": "demo"},
            {"name": "payments", "alias": "payments", "defaultSchema": "bronze"},
            {"name": "vietjet", "alias": "vietjet", "defaultSchema": "bronze"},
            {"name": "hdbank", "alias": "hdbank", "defaultSchema": "bronze"},
            {"name": "vikki", "alias": "vikki", "defaultSchema": "bronze"},
            {"name": "crm", "alias": "crm", "defaultSchema": "bronze"},
        ]
        environment = {
            "UNITY_CATALOGS": json.dumps(catalogs),
            "UNITY_CATALOG_ENDPOINT": "http://unity-catalog:8080",
            "S3_ENDPOINT": "rustfs:9000",
        }
        connection = RecordingConnection()

        with patch.dict(os.environ, environment, clear=True):
            aliases = configure_unity_catalogs(
                connection,  # type: ignore[arg-type]
                "user-token",
            )

        self.assertEqual(
            aliases, ["unity", "payments", "vietjet", "hdbank", "vikki", "crm"]
        )
        attach_statements = [
            statement for statement in connection.statements if "ATTACH" in statement
        ]
        self.assertEqual(len(attach_statements), 6)
        self.assertIn("ATTACH 'unity' AS \"unity\"", attach_statements[0])
        self.assertIn("DEFAULT_SCHEMA 'demo'", attach_statements[0])
        self.assertIn("ATTACH 'payments' AS \"payments\"", attach_statements[1])
        self.assertIn("DEFAULT_SCHEMA 'bronze'", attach_statements[1])
        for index, catalog in enumerate(
            ["vietjet", "hdbank", "vikki", "crm"], start=2
        ):
            self.assertIn(
                f"ATTACH '{catalog}' AS \"{catalog}\"", attach_statements[index]
            )
            self.assertIn("DEFAULT_SCHEMA 'bronze'", attach_statements[index])


if __name__ == "__main__":
    unittest.main()
