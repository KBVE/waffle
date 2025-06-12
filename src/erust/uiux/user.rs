#[derive(Debug, Clone, Default)]
pub struct User {
    pub id: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub is_authenticated: bool,
}

impl User {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_id(&mut self, id: String) {
        self.id = Some(id);
    }

    pub fn set_email(&mut self, email: String) {
        self.email = Some(email);
    }

    pub fn set_username(&mut self, username: String) {
        self.username = Some(username);
    }

    pub fn authenticate(&mut self) {
        self.is_authenticated = true;
    }

    pub fn logout(&mut self) {
        self.id = None;
        self.email = None;
        self.username = None;
        self.is_authenticated = false;
    }

    pub fn is_logged_in(&self) -> bool {
        self.is_authenticated
    }
}
