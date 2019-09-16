trait Consumer {
    fn consume_trk() {}
    fn consume_pkt_raw(pkt_raw: &[u8]) {}
    fn consume_pkt() {}
    fn consume_frm() {}
}

trait WithConsumers {
    fn consumers() -> Vec<Consumer> {}
    fn add_consumer(consumer: Consumer) {
        it.consumers().push(consumer)
    }
}

trait Producer: WithConsumers {
    fn consumers() -> Vec<Consumer> {}

    fn produce_trk(&self, trk: &[u8]) {
        for consumer in self.consumers() {
            consumer.consume_trk()
        }
    }

    fn produce_pkt_raw(&self, pkt_raw: &[u8]) {
        for consumer in self.consumers() {
            consumer.consume_pkt_raw(pkt_raw)
        }
    }

    fn produce_pkt(&self) {
        for consumer in self.consumers() {
            consumer.consume_pkt()
        }
    }

    fn consume_frm(&self) {
        for consumer in self.consumers() {
            consumer.consume_frm()
        }
    }
}

trait EntryPoint: Producer {}
trait Filter: Producer + Consumer {}
trait EndPoint: Consumer {}
