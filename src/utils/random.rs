use uuid::Uuid;

pub fn random_device_id() -> String {
    let id = Uuid::new_v4();
    id.to_string()
}
