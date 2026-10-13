"""Transform Vikki bronze events into typed silver tables."""

from common import BIGINT, BOOLEAN, DATE, STRING, TIMESTAMP, Column, JobConfig, Table, run


if __name__ == "__main__":
    run(
        JobConfig(
            source="vikki",
            tables=(
                Table(
                    name="customers",
                    event_type="com.vikki.t24.customer.snapshot",
                    columns=(
                        Column("customer_id", STRING, "Vikki customer identifier"),
                        Column("full_name", STRING, "Customer full name"),
                        Column("date_of_birth", DATE, "Customer date of birth"),
                        Column("national_id", STRING, "Customer national identity number"),
                        Column("phone", STRING, "Customer phone number"),
                        Column("email", STRING, "Customer email address"),
                        Column("address", STRING, "Customer postal address"),
                        Column("city", STRING, "Customer city"),
                        Column("kyc_status", STRING, "Know-your-customer verification status"),
                        Column("created_at", TIMESTAMP, "Timestamp when the customer was created"),
                    ),
                ),
                Table(
                    name="accounts",
                    event_type="com.vikki.t24.account.snapshot",
                    columns=(
                        Column("account_id", STRING, "Vikki account identifier"),
                        Column("customer_id", STRING, "Identifier of the account owner"),
                        Column("account_type", STRING, "Type of account"),
                        Column("currency", STRING, "ISO currency code for the account"),
                        Column("balance", BIGINT, "Account balance in the smallest whole currency unit"),
                        Column("wallet_is_default", BOOLEAN, "Whether this is the customer's default wallet"),
                        Column("opened_at", DATE, "Date when the account was opened"),
                        Column("status", STRING, "Current account status"),
                    ),
                ),
                Table(
                    name="wallet_transactions",
                    event_type="com.vikki.t24.wallet.transaction.completed",
                    columns=(
                        Column("transaction_id", STRING, "Wallet transaction identifier"),
                        Column("account_id", STRING, "Account associated with the transaction"),
                        Column("occurred_at", TIMESTAMP, "Timestamp when the transaction occurred"),
                        Column("direction", STRING, "Transaction direction, such as debit or credit"),
                        Column("category", STRING, "Transaction spending category"),
                        Column("merchant", STRING, "Merchant name"),
                        Column("amount_vnd", BIGINT, "Transaction amount in Vietnamese dong"),
                        Column("status", STRING, "Current transaction status"),
                        Column("reference", STRING, "External transaction or payment reference"),
                    ),
                ),
            ),
        )
    )
