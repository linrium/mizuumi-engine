import os
import sys
import types
import unittest
import urllib.parse
from unittest.mock import MagicMock, patch


pyspark = types.ModuleType("pyspark")
pyspark_sql = types.ModuleType("pyspark.sql")
pyspark_functions = types.ModuleType("pyspark.sql.functions")
pyspark_sql.DataFrame = object
pyspark_sql.Observation = object
pyspark_sql.SparkSession = object
pyspark_functions.col = lambda name: name
pyspark_functions.count = lambda value: value
pyspark_functions.lit = lambda value: value
sys.modules.setdefault("pyspark", pyspark)
sys.modules.setdefault("pyspark.sql", pyspark_sql)
sys.modules.setdefault("pyspark.sql.functions", pyspark_functions)

from common import (
    Column,
    JobConfig,
    STRING,
    TIMESTAMP,
    Table,
    controlplane_access_token,
    controlplane_request,
    contract_document,
    create_table,
    validate_dataframe,
)


class FakeType:
    def __init__(self, value: str):
        self.value = value

    def simpleString(self) -> str:
        return self.value


class FakeField:
    def __init__(self, name: str, data_type: str, nullable: bool = True):
        self.name = name
        self.dataType = FakeType(data_type)
        self.nullable = nullable


class FakeRows:
    def __init__(self, fields):
        self.schema = types.SimpleNamespace(fields=fields)


class FakeSpark:
    def __init__(self):
        self.statements = []

    def sql(self, statement: str):
        self.statements.append(statement)


class ContractTests(unittest.TestCase):
    def setUp(self):
        self.table = Table(
            name="contacts",
            event_type="com.crm.contact.snapshot",
            columns=(Column("contact_id", STRING, "CRM contact identifier"),),
        )
        self.config = JobConfig(source="crm", tables=(self.table,))
        self.document = contract_document(self.config, self.table, "crm.silver.contacts")

    def test_contract_is_stable_and_includes_ingestion_columns(self):
        again = contract_document(self.config, self.table, "crm.silver.contacts")
        self.assertEqual(self.document["id"], again["id"])
        self.assertEqual(self.document["apiVersion"], "v3.1.0")
        self.assertEqual(self.document["version"], "1.1.0")
        self.assertEqual(self.document["team"]["members"][1]["role"], "Data Steward")
        self.assertEqual(self.document["customProperties"][0]["value"], "VN")
        self.assertEqual(self.document["slaProperties"][0]["property"], "frequency")
        self.assertEqual(
            self.document["schema"][0]["quality"][0]["metric"], "rowCount"
        )
        self.assertEqual(
            self.document["schema"][0]["properties"][0]["quality"][0]["metric"],
            "nullValues",
        )
        self.assertEqual(
            [item["name"] for item in self.document["schema"][0]["properties"]],
            ["contact_id", "kafka_ts", "ingest_ts"],
        )

    def test_dataframe_must_match_contract_order_and_types(self):
        contract = {"document": self.document}
        matching = FakeRows(
            [
                FakeField("contact_id", "string"),
                FakeField("kafka_ts", "timestamp"),
                FakeField("ingest_ts", "timestamp"),
            ]
        )
        validate_dataframe(matching, contract)

        drifted = FakeRows(
            [
                FakeField("contact_id", "bigint"),
                FakeField("kafka_ts", "timestamp"),
                FakeField("ingest_ts", "timestamp"),
            ]
        )
        with self.assertRaisesRegex(RuntimeError, "does not satisfy"):
            validate_dataframe(drifted, contract)

    def test_table_creation_stamps_contract_properties(self):
        spark = FakeSpark()
        contract = {
            "id": self.document["id"],
            "version": "1.0.0",
            "document_hash": "document-hash",
            "schema_hash": "schema-hash",
        }
        create_table(spark, "crm.silver.contacts", self.table, contract)

        self.assertEqual(len(spark.statements), 1)
        self.assertIn("TBLPROPERTIES", spark.statements[0])
        self.assertIn("mizuumi.odcs.documentHash", spark.statements[0])

    def test_exchanges_the_dedicated_controlplane_client_credentials(self):
        response = MagicMock()
        response.__enter__.return_value.read.return_value = (
            b'{"access_token":"temporary-token"}'
        )
        env = {
            "KEYCLOAK_TOKEN_URL": "https://auth.example/token",
            "KEYCLOAK_CA_CERT": "/tmp/keycloak-ca.crt",
            "CONTROLPLANE_CLIENT_ID": "spark-controlplane",
            "CONTROLPLANE_CLIENT_SECRET": "secret",
        }

        with (
            patch.dict(os.environ, env, clear=True),
            patch("common.os.path.isfile", return_value=True),
            patch("common.ssl.create_default_context"),
            patch("common.urllib.request.urlopen", return_value=response) as urlopen,
        ):
            self.assertEqual(controlplane_access_token(), "temporary-token")

        token_request = urlopen.call_args.args[0]
        token_fields = urllib.parse.parse_qs(token_request.data.decode("ascii"))
        self.assertEqual(token_fields["grant_type"], ["client_credentials"])
        self.assertEqual(token_fields["client_id"], ["spark-controlplane"])

    def test_controlplane_requests_include_the_temporary_bearer_token(self):
        response = MagicMock()
        response.__enter__.return_value.read.return_value = b'{"ok":true}'
        with (
            patch.dict(
                os.environ,
                {"CONTROLPLANE_URL": "http://controlplane:8080"},
                clear=True,
            ),
            patch("common.controlplane_access_token", return_value="temporary-token"),
            patch("common.urllib.request.urlopen", return_value=response) as urlopen,
        ):
            result = controlplane_request("POST", "/api/test", {"value": 1})

        self.assertEqual(result, {"ok": True})
        api_request = urlopen.call_args.args[0]
        self.assertEqual(api_request.get_header("Authorization"), "Bearer temporary-token")


if __name__ == "__main__":
    unittest.main()
