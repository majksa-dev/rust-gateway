#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
    Days,
}

impl TimeUnit {
    pub fn convert(&self, amount: usize, unit: TimeUnit) -> usize {
        let seconds = match self {
            TimeUnit::Seconds => amount,
            TimeUnit::Minutes => amount * 60,
            TimeUnit::Hours => amount * 3600,
            TimeUnit::Days => amount * 86400,
        };
        match unit {
            TimeUnit::Seconds => seconds,
            TimeUnit::Minutes => seconds / 60,
            TimeUnit::Hours => seconds / 3600,
            TimeUnit::Days => seconds / 86400,
        }
    }
}

#[derive(Debug)]
pub struct Time {
    pub amount: usize,
    pub unit: TimeUnit,
}

impl Time {
    pub fn convert(&self, unit: TimeUnit) -> Time {
        Time {
            amount: self.unit.convert(self.amount, unit),
            unit,
        }
    }
}

#[derive(Debug)]
pub struct Frequency {
    pub amount: usize,
    pub interval: Time,
}
