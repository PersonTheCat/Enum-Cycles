
/// This trait defines all of the necessary procedures which enable enum values
/// to behave as states of a single type. These types can be nested, for example,
/// to represent a hierarchy of one larger state, e.g.
/// ```ignore
///     #[auto]
///     #[derive(Clone, EnumState)]
///     enum AppFocus {
///         MainWindow(MainFocus),
///         OtherWindow
///     }
///
///     #[default(StatsTab)]
///     #[derive(Clone, EnumState)]
///     enum MainFocus {
///         StatsTab,
///         GraphsTab,
///         InfoTab
///     }
/// ```
/// States can then be scrolled--or *cycled*--through by calling `#next()`
/// or another such method without the use of indices or strings, e.g.
/// ```ignore
///     struct App {
///         focus: AppFocus
///     }
///
///     fn next_tab(app: &mut App) {
///         if let MainWindow(ref mut focus) = app.focus {
///             focus.next();
///         }
///     }
/// ```
/// In addition, `EnumState` will provide a couple of convenient functions
/// for retrieving stats about the current state and / or potential states.
/// this includes data about the current state's relative index, name, and
/// even the complete set of (default) values and names.
pub trait EnumState: Sized + Clone + 'static {

    /// Stores the name of each variant in the enum.
    const _NAMES: &'static [&'static str];

    /// Stores the default value of each variant.
    const _VALUES: &'static [Self];

    /// The default value for this enum.
    const _DEFAULT: Self;

    /// The first value listed in the enum.
    const _FIRST: Self;

    /// The last value listed in the enum.
    const _LAST: Self;

    /// The number of elements in the enum.
    const _SIZE: usize;

    /// Skips the current state forward one value.
    fn next(&mut self) {
        self.skip(1);
    }

    /// Skips backward to the previous value.
    fn previous(&mut self) {
        self.skip_backward(1);
    }

    /// Increments the state by the input `num`. If the resulting
    /// index would be greater than the maximum possible, this
    /// function skips to the last possible state. If the current
    /// state *is* the last possible state, it will skip to the
    /// first possible state.
    fn skip(&mut self, num: usize) {
        let mut index = self.index();
        let size = Self::size();
        let max = size - 1;
        let sum = index + num;

        if index == max {
            index = 0;
        } else if sum > max {
            index = max;
        } else {
            index = sum % size;
        }
        *self = Self::from_index(index).unwrap();
    }

    /// Decrements the state by the input `num`. If the resulting
    /// index would be less than zero, this function skips to the
    /// first possible state. If the current state *is* the first
    /// possible state, it will skip to the last possible state.
    fn skip_backward(&mut self, num: usize) {
        let mut index = self.index();
        let size = Self::size();

        if num == 0 {
            return;
        } else if index > num - 1 {
            index -= num;
        } else if index == 0 {
            index = size - 1;
        } else {
            index = 0;
        }
        *self = Self::from_index(index).unwrap();
    }

    /// Attempts to retrieve the default value for the variant
    /// at the given index.
    fn from_index(i: usize) -> Option<Self> {
        if i < Self::_SIZE {
            Some(Self::_VALUES[i].clone())
        } else {
            None
        }
    }

    /// Yields the set of possible names for this enum.
    fn names() -> &'static [&'static str] {
        Self::_NAMES
    }

    /// Yields the set of default values for this enum.
    fn values() -> &'static [Self] {
        Self::_VALUES
    }

    /// Yields the default value for this enum.
    fn default() -> Self {
        Self::_DEFAULT
    }

    /// Yields the first value in the enum.
    fn first() -> Self {
        Self::_FIRST
    }

    /// Yields the last value in the enum.
    fn last() -> Self {
        Self::_LAST
    }

    /// Yields the number of elements in the enum.
    fn size() -> usize {
        Self::_SIZE
    }

    /// Determines the index of the current state.
    fn index(&self) -> usize;

    /// Determines the name of the current state.
    fn name(&self) -> &'static str;
}