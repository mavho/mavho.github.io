
In distributed message process, there are different types of processing/delivery semantics used to handle the processing of diferent messages under different failure modes and use cases. Kafka provides three delivery semantics out of the box, which can be configured directly onto the configuration - all three with different use cases. There are
    - *at most once* semantics - which offers the highest throughput of sent messages - but the least durability of written messages.
    - *at least once* semantics - offers strong durability of written messages - messages are always written. However under failures messages could be duplicated.
    - *exactly once* semantics - offers the strongest durability of written messages - messages are written exactly once - even under failure.

*at most once* and *at least once* semantics are the easiest to configure - both just require fine tuning various settings on the Kafka producer/consumer. However - for *exactly  once* semantics - it requires fine tuning settings on both the kafka producer **and** consumer, as well as writing application side code for transactions that enable *exactly once* **processing and delivery** of messages.

> side note - these semantics apply to both the **delivery** of a message by a producer - and the **receipt/processing** of a message from the consumer. These semantics can be mixed and matched with each other if different guarantees are needed for the **flow** of delivery -> processing work cycle. *exactly once* semantics is able to offer the guarantees on **both** the producer and consumer.

#  At Most Once Semantics and At Least Once Semantics
Before getting to *exactly once* semantics - we'll go over the two basic settings on both the producer and consumer side.

- *at most once semantics*:
    - **Delivery:** Messages are delivered to the leader at most once. Configuring *at most once* means that when a producer sends a message - it will only send the message at most one time. This means that the producer doesn't have to wait for an acknowledgement from the leader - and can process/send messages asynchronously without handling any retries or acknowledgements - providing the fastest throughput and latency of all the semantics. However in cases of a failure - messages can be lost.
        - to configure *at most once semnatics*
            - set `retries=0`
            - set `acks=0`, since you're not waiting for any acknowledgement from the leader.
            - set `enable.idempotence=false` - as setting it to true will conflict with the other settings. (Kafka will automatically set this for you)
        - In summary - this can be seen as *fire and forget*
    - **Processing:** Messages are read by the leader at most once - meaning once the consumer has polled the messages - it commits it's offset (place in the partition) - and then processes the message. If the consumer crashes after commiting it's offset - but before processing the messages - the consumer that takes over will continue where the other consumer left off.
        - to configure *at most once semantics* - you usually want to commit offsets before processing messages - and that can be done within the application code.
            - set `enable.auto.commit=false`, and commit right after you poll/batch the messages.
        - however it's best to let the consumer handle the auto commit - unless you need **exact** *at most once* semantics.
            - set `enable.auto.commit=true` - however this could process messages multiple times. This could happen when the consumer has consumed a message - but processed them before commiting. Therefore when a consumer dies while processing the messages - but before committing them - the consumer that takes over will reprocess those messages.

- *at least once semantics*:
    - **Delivery:** Messages are delivered to the broker at least once. Configuring *at least once once* means when a producer sends a message, it will wait for the broker to acknowledge that message. Once the broker acknowledges that message - it means that the message has been written to that broker (topic) - and depending on the `ack` setting, been persisted to that topic.
        - to configure *at least once semantics*
            - set `retries` to the default settings (which is essentially unlimited retries)
            - set `delivery.timeout.ms` to an upper bound in `ms` that directly correlates to `retries`
            - most importantly set `acks=all`. If `acks=0` - messages could be lost because the producer can send a message to the broker, but if the message fails to be delivered - or the broker failed, the producer will not wait for any acknowledgement - and the `retries` setting will not be invoked. Additionally - if `acks=1`, then the leader will acknowledge that the message has been written without replicating it to the followers. Therefore a message could be lost if the leader acknowledges the message - but fails to replicate it to the followers then dies. When `acks=all` - in times of system failure - we know that a message has been successfully written/replicated to a topic because the leader will only acknowledge a successful message IF all replicas have acknowledged it.
        > Optionally set `enable.idempotence=true`. This setting is generally already set to true - and enables that the producer will have only one copy the message written to the broker. This also effectively sets the delivery semantics to *exactly once* on the producer's side. We will go over this more in the *exactly once* section.
        - therefore with this configuration, messages are delivered one or more times. In the chance of a system failure - messages are never lost - but may be delivered more than once. With idempotence enabled - you've also configured *exactly once* on the producer side.
    - **Processing:** Messages are processed by the consumer at least once. In order to configure *at least once* for the consumer you'll have to be stricter on how you process messages from the topic-partition and how you commit offsets. To achieve this semantic, the consumer reads a set of messages, processes them, then commits the offsets.
        - to configure *at least once* semantics - you usually want to commit offsets after processing messages. Therefore in the case of failing a batch - messages can be reconsumed at the same offset - or if the consumer fails before committing - it can start again at the same place. This means that messages will be processed at least once.
            - set `enable.auto.commit=false`, and commit offsets after you process the messages.


# *At Most Once* and *At Least Once* failure scenarios
