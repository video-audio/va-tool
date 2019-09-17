// trait EntryPoint: Producer {}
// trait Filter: Producer + Consumer {}
// trait EndPoint: Consumer {}

pub struct Consumers(Vec<Box<dyn Consumer>>);

impl Default for Consumers {
    fn default() -> Self {
        Consumers(Vec::new())
    }
}

pub trait Consumer {
    fn consume_trk(&self) {}
    fn consume_pkt_raw(&self, _: &[u8]) {}
    fn consume_pkt(&self) {}
    fn consume_frm(&self) {}
}

pub trait Producer {
    fn consumers(&self) -> &Consumers;
    fn consumers_mut(&mut self) -> &mut Consumers;

    fn add_consumer(&mut self, consumer: Box<dyn Consumer>) {
        self.consumers_mut().0.push(consumer)
    }

    fn produce_trk(&self) {
        for consumer in self.consumers().0.iter() {
            consumer.consume_trk()
        }
    }

    fn produce_pkt_raw(&self, pkt_raw: &[u8]) {
        for consumer in self.consumers().0.iter() {
            consumer.consume_pkt_raw(pkt_raw)
        }
    }

    fn produce_pkt(&self) {
        for consumer in self.consumers().0.iter() {
            consumer.consume_pkt()
        }
    }

    fn consume_frm(&self) {
        for consumer in self.consumers().0.iter() {
            consumer.consume_frm()
        }
    }
}

pub struct Filter {
    consumers: Consumers,
}

impl Default for Filter {
    fn default() -> Self {
        Filter {
            consumers: Default::default(),
        }
    }
}

impl Producer for Filter {
    fn consumers(&self) -> &Consumers {
        &self.consumers
    }

    fn consumers_mut(&mut self) -> &mut Consumers {
        &mut self.consumers
    }
}

impl Consumer for Filter {
    fn consume_trk(&self) {
        self.produce_trk()
    }
}
