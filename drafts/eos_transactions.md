
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
        > also in practice you set `retries` to a high number, and set `request.delivery.timeout` to set the duration a producer will retry a message.
    - **Processing:** Messages are processed by the consumer at least once. In order to configure *at least once* for the consumer you'll have to be stricter on how you process messages from the topic-partition and how you commit offsets. To achieve this semantic, the consumer reads a set of messages, processes them, then commits the offsets.
        - to configure *at least once* semantics - you usually want to commit offsets after processing messages. Therefore in the case of failing a batch - messages can be reconsumed at the same offset - or if the consumer fails before committing - it can start again at the same place. This means that messages will be processed at least once.
            - set `enable.auto.commit=false`, and commit offsets after you process the messages.

# Failure Scenarios

Let's go over some common failure scenarios that can occur during message delivery to a topic.

## A) Message Ack Fails, But is written to the log

A producer could send over a message to the leader of the topic partition, and the leader is able to successfuly write and replicate that record to the log. The leader then sends an acknowledgement back to the producer, but that acknowledgement is lost for what ever reason. If your producer is configured with retries then the producer will retry the produce message, and the message will be written twice into the topic partition.

## B) Producer Fails While Processing Messages

A producer could be in the process of producing many messages to the topic partition, however it crashes half way through. However based on the producer's application code what should it do? Should it restart from where it left off - which could reintroduce duplicates? If you're going for *at least once* delivery semantics, that would be the way to go, or if you're going for *at most once* delivery semantics - you would pick up from where it crashes.

## C) Consumer Fails While Processing Messages

A consumer could fail while in the process of consuming messages from a topic partition. In the event that the consumer crashes before committing their offset - when the consumer is started up again then it will consume the same records again - which introduces duplicates on the consumer side.


As you can see, failures can happen during the producer, broker, and consumer side. Depending on how you've coded and configured the producer and consumer, you'll get different delivery and processing semantics on both ends. Depending on the requirements of your architecture - you may be satisfied to leave it off on *at least/most once* semantics - however some systems require stronger delivery semantics - such as *exactly once* in both it's delivery and processing guarantees in face of failures.


# Exactly Once Semantics (EOS)

Simply put, in a Kafka architecture *exactly once* means that a message is delivered *exactly once* to the broker - **and** processed *exactly once* by the consumer - even in the event of failure. This not only requires the correct Kafka configuration on the producer and consumer side, but also application logic on the producer and consumer code.

We mentioned earlier [insert link here] that the producer can actually implement *exactly once* delivery by setting `acks=all`, settings retries, and `enable.idempotence=true`. This ensures that a message has been delivered to a topic partition exactly one time. This helps solve scenario A) and broker failures. However this doesn't solve issues B) - as the Kafka producer API does not know the inner workings of your code - and doesn't solve issue C) since the consumer could fail and re process the messages again.

This is where Kafka transactions steps in.


# Kafka Transactions

Transactions in Kafka allow atomic multi-partition writes which will be reflected onto the consumer side. What this means is that a producer can create a transaction, write to many topics, and then commit or abort that transaction. If the producer committed that transaction - then the consumers reading from those topics will be able to read those messages. If the producer aborted that transaction - then the consumers reading from those topics will **not** read thoses messages.

Ok, so a producer can now perform atomic writes (either all messages are readable or none are). So how exactly does that help for *exactly once* semantics?

In many Kafka workflows, you'll have a process consuming messages from a kafka topic, performing some operation on them - and then producing output messages back into another kafka topic. This is called a *read - process - write* pattern. Processes that employ this pattern can send their consumed offsets back to the `__consumed_offsets` topic as part of the transaction process - which durably writes their position a topic partition if the transaction is committed. If - for some reason - that a process crashes before sending their consumed offsets to the transaction manager and committing, then the downstream consumers won't be able to see that process's produces (from a failed transaction) - which is what EOS provides to the consumed and produced messages. Messages that are *read - processed - produced* are wrapped up atomically and are only processed once. Essentially produces that are sent/created due to a batch processing of various messages are wrapped atomically with each other.

Let's run over an example: let's say there's an input topic `tpIn` and an output topic `tpOut`. We have a process that reads in messages from `tpIn`, performs some function on that message, and produces out to `tpOut`. Message A' is marked consumed when offset X(A') is sent to the offsets topic and then committed. Message B' is mapped to the successful output of the transaction - and will only be seen by downstream consumers if the transaction was able to be completed. In fact - the producer could've sent to multiple different topic-partitions within the same transaction - and they'll only be seen by downstream consumers if the transaction was committed.

> It's important to note - downstream consumers will only read committed if `isolation.level=read_committed` is set. If it's set to `read_uncommitted` consumers will see aborted messages and messages that are still ongoing in a transaction.
> It is also important to note that this *read - process -write* pattern only applies to within the Kafka Ecosystem - meaning the write portion of the pattern only applies to Kafka produces and doesn't apply to external systems processing.  

## Transactional.id

Kafka introduces a `transactional.id` to help combat a zombie. Let's say that a consumer is lagging behind or we think that it has failed, and we bring up another consumer to perform the task. However now we have two processes that could be potentially producing multiple duplicate messages. Transactional id's help prevent this because the producer will have to register it's ID to the transactional coordinator on startup. When the same `transactional.id` is seen again, the coordinator will increase it's `epoch` - which the coordinator uses to mark which is the most recent producer. Any transaction operation with an older epoch is fenced and the application code could die gracefully.

## Transactions Workflow
To help facilitate transactions, Kafka introduces a Transaction Coordinator (TXN Coordinator), a broker chosen to help facilitate requests from the Producers and other brokers.
1. Producer & Transaction Coordinator Interaction
    - upon startup the producer chooses a `transactional.id` and initiates a transaction with the TXN Coordiantor. 

2. Txn Coordinator
    - The TXN Coordinator keeps track of a transaction log - a durable kafka topic that keeps track of transaction state amongst many different transactions. The producer will send updates to the coordinator to write to this log.
3. Producer
    - Producer begins the transaction, which will send a request to the TXN Coordinator to start a transaction.
    - Producer can send many different messages to the output topic-partitions
    - After the producer processes the messages and wants to commit, the producer will send it's `consumed_offsets` as part of the transaction to the transaction coordinator.
    - If all steps were successful - the Producer can commit - marking the transaction as complete - or abort in case of an error.
4. Txn Coordinator to Topic/Partition Interaction
    - When the producer commits/aborts - the coordinator starts a two phase commit process.
        - 1st phase: The coordinator will update the transaction log to (prepare commit/abort) which will execute no matter what. This will durably mark that a transaction needs to bet committed or aborted. Therefore if the coordinator fails - another one will be able to take it's place.
        - 2nd phase: The coordinator writes commit markers to topic/partitions that are apart of the transactions.
    - These commit markers are then used by consumers in `read_committed` mode to either use or filter out aborted messages. Once these markers have finished - the coordinator marks the transaction as done.

# Implement EOS Semantics using Kafka Transactions with Rust
Now that we have a basic understanding about Kafka EOS - I'll show you how to use them using Rust as an example. Hopefully this will better help you understand what's going during the *read-process-write* interactions within transactions.

## Configuration
To configure a transactional producer, you'll need to set
    - `enable.idempotence=true`
    - Set retries to a high number (leave default)`retries=2147483647`
    - `acks=all`
    - `transactional.id` - set a transactional ID for this process instance.
To configure a transaction aware consumer, you should set
    -`isolation.level=read_committed`

The transactional producer and consumer should sit in the same process space in order for the transactional producer to be able to send the consumed offsets as part of the transaction

