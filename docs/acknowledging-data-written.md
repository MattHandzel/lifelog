```
Vector has the capability of allowing clients to verify that data has been delivered to destination sinks. This is called end-to-end acknowledgement.
Design

When a participating source receives an event, or batch of events, it can optionally create a batch notifier for those events. The batch notifier has two parts: one part stays with the source, and the other part is attached to the events. When the events reach their destination sink and are processed by the sink, Vector captures the status of the response from the downstream service and uses it to update the batch notifier. By doing so, we can indicate whether an event was successfully processed or not.

Additionally, Vector ensures that the batch notifier for an event is always updated, whether or not the event made it to a sink. This ensures that if an event is intentionally dropped (for example, by using a filter transform) or even unintentionally dropped (maybe Vector had a bug, uh oh!), we still update the batch notifier to indicate the processing status of the event.

Meanwhile, the source will hold on to the other half of the batch notifiers that it has created, and is notified when a batch notifier is updated. Once notified, a source will propagate that batch notifier status back upstream: maybe this means responding with an appropriate HTTP status code (200 vs 500, etc) if the events came from an HTTP request, or acknowledging the event directly, such as when using the kafka or aws_sqs sources, which have native support for acknowledging messages.
```
