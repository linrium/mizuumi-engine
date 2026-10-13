"""Transform VietJet bronze events into typed silver tables."""

from common import BIGINT, DATE, STRING, TIMESTAMP, Column, JobConfig, Table, run


if __name__ == "__main__":
    run(
        JobConfig(
            source="vietjet",
            tables=(
                Table(
                    name="passengers",
                    event_type="com.vietjet.passenger.snapshot",
                    columns=(
                        Column("passenger_id", STRING, "VietJet passenger identifier"),
                        Column("full_name", STRING, "Passenger full name"),
                        Column("date_of_birth", DATE, "Passenger date of birth"),
                        Column("national_id", STRING, "Passenger national identity number"),
                        Column("phone", STRING, "Passenger phone number"),
                        Column("email", STRING, "Passenger email address"),
                        Column("loyalty_tier", STRING, "Passenger loyalty program tier"),
                        Column("created_at", TIMESTAMP, "Timestamp when the passenger was created"),
                    ),
                ),
                Table(
                    name="bookings",
                    event_type="com.vietjet.booking.recorded",
                    columns=(
                        Column("booking_id", STRING, "VietJet booking identifier"),
                        Column("passenger_id", STRING, "Passenger associated with the booking"),
                        Column("pnr", STRING, "Passenger name record locator"),
                        Column("booked_at", TIMESTAMP, "Timestamp when the booking was made"),
                        Column("departure_at", TIMESTAMP, "Scheduled departure timestamp"),
                        Column("origin", STRING, "Origin airport IATA code"),
                        Column("destination", STRING, "Destination airport IATA code"),
                        Column("fare_class", STRING, "Booked fare class"),
                        Column("fare_vnd", BIGINT, "Booking fare in Vietnamese dong"),
                        Column("payment_method", STRING, "Payment method used for the booking"),
                        Column("payment_reference", STRING, "External payment reference"),
                        Column("status", STRING, "Current booking status"),
                    ),
                ),
            ),
        )
    )
