pub mod counter;

// Let's define the messages handled by our app. NOTE: it must derive `PartialEq`
#[derive(Debug, PartialEq)]
pub enum Msg {
    AppClose,
    Clock,
    DigitCounterChanged(isize),
    DigitCounterBlur,
    LetterCounterChanged(isize),
    LetterCounterBlur,
}

// Let's define the component ids for our application
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Id {
    DigitCounter,
    LetterCounter,
}
