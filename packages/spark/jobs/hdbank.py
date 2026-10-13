"""Transform HDBank bronze events into typed silver tables."""

from common import BIGINT, DATE, DECIMAL_5_2, INT, STRING, TIMESTAMP, Column, JobConfig, Table, run


if __name__ == "__main__":
    run(
        JobConfig(
            source="hdbank",
            tables=(
                Table(
                    name="customers",
                    event_type="com.hdbank.t24.customer.snapshot",
                    columns=(
                        Column("customer_id", STRING, "HDBank customer identifier"),
                        Column("full_name", STRING, "Customer full name"),
                        Column("date_of_birth", DATE, "Customer date of birth"),
                        Column("national_id", STRING, "Customer national identity number"),
                        Column("phone", STRING, "Customer phone number"),
                        Column("email", STRING, "Customer email address"),
                        Column("address", STRING, "Customer postal address"),
                        Column("city", STRING, "Customer city"),
                        Column("segment", STRING, "Customer business segment"),
                        Column("kyc_status", STRING, "Know-your-customer verification status"),
                        Column("created_at", TIMESTAMP, "Timestamp when the customer was created"),
                    ),
                ),
                Table(
                    name="loans",
                    event_type="com.hdbank.t24.loan.snapshot",
                    columns=(
                        Column("loan_id", STRING, "HDBank loan identifier"),
                        Column("customer_id", STRING, "Identifier of the borrower"),
                        Column("product", STRING, "Loan product type"),
                        Column("principal_vnd", BIGINT, "Original loan principal in Vietnamese dong"),
                        Column("annual_rate_pct", DECIMAL_5_2, "Annual interest rate as a percentage"),
                        Column("term_months", INT, "Contracted loan term in months"),
                        Column("outstanding_vnd", BIGINT, "Outstanding principal in Vietnamese dong"),
                        Column("originated_at", DATE, "Date when the loan originated"),
                        Column("status", STRING, "Current loan status"),
                    ),
                ),
                Table(
                    name="loan_repayments",
                    event_type="com.hdbank.t24.loan.repayment.completed",
                    columns=(
                        Column("repayment_id", STRING, "Loan repayment identifier"),
                        Column("loan_id", STRING, "Loan associated with the repayment"),
                        Column("installment_number", INT, "Sequential installment number"),
                        Column("due_date", DATE, "Contractual repayment due date"),
                        Column("paid_at", TIMESTAMP, "Timestamp when the repayment was paid"),
                        Column("principal_vnd", BIGINT, "Principal component paid in Vietnamese dong"),
                        Column("interest_vnd", BIGINT, "Interest component paid in Vietnamese dong"),
                        Column("status", STRING, "Current repayment status"),
                    ),
                ),
            ),
        )
    )
