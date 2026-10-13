"""Transform CRM bronze events into typed silver tables."""

from common import DATE, STRING, TIMESTAMP, Column, JobConfig, Table, run


if __name__ == "__main__":
    run(
        JobConfig(
            source="crm",
            tables=(
                Table(
                    name="contacts",
                    event_type="com.crm.contact.snapshot",
                    columns=(
                        Column("contact_id", STRING, "CRM contact identifier"),
                        Column("full_name", STRING, "Contact full name"),
                        Column("date_of_birth", DATE, "Contact date of birth"),
                        Column("national_id", STRING, "Contact national identity number when available"),
                        Column("phone", STRING, "Contact phone number"),
                        Column("email", STRING, "Contact email address"),
                        Column("city", STRING, "Contact city"),
                        Column("source", STRING, "Source through which the contact was acquired"),
                        Column("created_at", TIMESTAMP, "Timestamp when the contact was created"),
                        Column("updated_at", TIMESTAMP, "Timestamp when the contact was last updated"),
                    ),
                ),
                Table(
                    name="campaign_interactions",
                    event_type="com.crm.campaign.interaction.recorded",
                    columns=(
                        Column("interaction_id", STRING, "Campaign interaction identifier"),
                        Column("contact_id", STRING, "Contact associated with the interaction"),
                        Column("campaign_id", STRING, "Marketing campaign identifier"),
                        Column("channel", STRING, "Channel used for the interaction"),
                        Column("occurred_at", TIMESTAMP, "Timestamp when the interaction occurred"),
                        Column("outcome", STRING, "Recorded interaction outcome"),
                        Column("consent_status", STRING, "Contact consent status at interaction time"),
                    ),
                ),
            ),
        )
    )
