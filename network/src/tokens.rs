use mio::Token;

pub struct Tokens {
    count: usize
}

impl Tokens {
    pub fn new() -> Tokens {
        Tokens {
            count: 0
        }
    }

    pub fn next(&mut self) -> Token {
        // self.count().clone()
        let mut count = self.count;
        count = count + 1;

        self.count = count;

        Token(count)
    }
}

