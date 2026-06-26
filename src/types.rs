use juniper::GraphQLInputObject;

pub type Frequencies = Vec<FrequencyAndMagnitude>;

#[derive(GraphQLInputObject)]
pub struct FrequencyAndMagnitude {
    pub frequency: f64,
    pub magnitude: i32,
}

impl FrequencyAndMagnitude {
    pub fn is_valid(&self) -> bool {
        self.magnitude >= 0 && self.magnitude < 100
    }
}
