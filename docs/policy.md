## Policy

The server has a policy that allows it to make decisions for whatever decisions the server needs to make. These decisions include:

- When should I request data from the collectors?
- How much CPU usage should I use?
- How much network traffic should I use?
- For all data sources:
  - Of the avaliable transforms, what transform should I run at this instance.
- Should I create a backup, if so, of what data?
- Should I compress any data?
- Should I re-transform any data (suppose)
- Should i train any models of data
- Does the current query require these data sources that aren't transformed.

The policy struct looks like

```rs
fn policy_loop(system: System, policy: Policy) {
    loop {
        let state = system.get_state();
        let action = policy.get_action(state);

        // Perform the action
        tokio::task::spawn(async move {
            system.do_action(action).await;
            // Add to audit log
            system.add_audit_log(action).await;
        });
    }
}

impl Policy for Policy {
    fn get_action(state: SystemState) -> Action {

        // Logic to decide the action
        // For example, look at the history, what actions we are already doing

        // Look at the last time different maintenance actions were performed

        // See what the current client requests are

        action = // decide action here

    }
}

enum Action {
    RequestData,
    CompressData,
    TransformData,
    TrainModel,
    BackupData,
    ReTransformData,
    CreateBackup,
    SendData,
}
```
