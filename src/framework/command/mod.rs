use serenity::model::channel::Message;
use std::default::Default;

pub struct Command {
    //TODO write struct
    pub name: String,
    action: fn(&Message, &String) -> bool,
    owner: bool,
    server: bool,
}

impl Default for Command {
    fn default() -> Command {
        Command {
            name: String::new(),
            action: |_,_| false,
            owner: false,
            server: false,
        }
    }
}

impl Command {
    pub fn new(name: String, action: fn(&Message, &String) -> bool) -> Command {
        Command {
            name,
            action,
            owner: false,
            server: false,
        }
    }

    pub fn execute(&self, m: &Message, str: &String) -> bool {
        (self.action)(m, str)
    }

    pub fn owner_only(mut self, state: bool) -> Command {
        self.owner = state;
        self
    }
    
    pub fn server_only(mut self, state: bool) -> Command {
        self.server = state;
        self
    }
}
