from __future__ import annotations

import json
from confluent_kafka import Consumer, Producer
from confluent_kafka.admin import AdminClient, NewTopic


class RedpandaClient:
    def __init__(self, bootstrap_servers: str = "localhost:9092") -> None:
        self._bootstrap_servers = bootstrap_servers
        self._admin = AdminClient({"bootstrap.servers": bootstrap_servers})

    def produce(self, topic: str, key: str, value: dict) -> None:
        producer = Producer({"bootstrap.servers": self._bootstrap_servers})
        producer.produce(
            topic,
            key=key.encode(),
            value=json.dumps(value).encode(),
        )
        producer.flush()

    def consume(self, topic: str, group_id: str, timeout: float = 1.0) -> list[dict]:
        consumer = Consumer({
            "bootstrap.servers": self._bootstrap_servers,
            "group.id": group_id,
            "auto.offset.reset": "earliest",
        })
        consumer.subscribe([topic])
        messages = []
        try:
            while True:
                msg = consumer.poll(timeout=timeout)
                if msg is None:
                    break
                if msg.error():
                    break
                messages.append(json.loads(msg.value().decode()))
        finally:
            consumer.close()
        return messages

    def create_topic(self, name: str, partitions: int = 3, replication_factor: int = 1) -> None:
        futures = self._admin.create_topics([
            NewTopic(name, num_partitions=partitions, replication_factor=replication_factor)
        ])
        futures[name].result()
