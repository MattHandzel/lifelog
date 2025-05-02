pub trait Policy {
    type StateType;
    type ActionType;
    fn get_action(&self, state: &Self::StateType) -> Self::ActionType;
}
