# Enum-Cycles
 A set of tools for interacting with enum types as status indicators

# Traits

## EnumState

This trait defines all of the necessary procedures which enable enum values
to behave as states of a single type. These types can be nested, for example,
to represent a hierarchy of one larger state, e.g.
```rust
    #[auto]
    #[derive(Clone, EnumState)]
    enum AppFocus {
        MainWindow(MainFocus),
        OtherWindow
    }
    
    #[default(StatsTab)]
    #[derive(Clone, EnumState)]
    enum MainFocus {
        StatsTab,
        GraphsTab,
        InfoTab
    }
```
States can then be scrolled--or *cycled*--through by calling `#next()`
or another such method without the use of indices or strings, e.g.
```rust
    struct App {
        focus: AppFocus
    }
    
    fn next_tab(app: &mut App) {
        if let MainWindow(ref mut focus) = app.focus {
            focus.next();
        }
    }
```
In addition, `EnumState` will provide a couple of convenient functions
for retrieving stats about the current state and / or potential states.
this includes data about the current state's relative index, name, and
even the complete set of (default) values and names.

## Deriving `EnumState`
`EnumState` can be derived using the standard `#[derive]` syntax, provided 
an implementation of `Clone` also be present. This macro supports four
attributes: `default`, `auto`, `first`, and `last`, which have the following
indications:

### `default`

When this token is placed at the top level (before the `enum` token),
it defines the default variant of the enum as a whole. For example,
use `#[default(Numbers::One)]` to indicate that `Numbers::One` is the
default state of the supplied enum.

When this token is placed at the variant level (just before the variant
itself), it defines the default input to forward into that variant. For
example, use `#[default(x)]` to indicate that `x` is the default value
for the variant's fields, separated by commas. `x` must be a constant
expression.

e.g.
```rust
    #[default(Numbers::One)]
    #[derive(Clone, EnumState)
    enum Numbers {
        One,
        #[default(Inner::Left)]
        Two(Inner)
    }

    #[derive(Clone, EnumState)]
    enum Inner {
        Left,
        Right
    }
```

### `first`

When this token is placed at the top level, it informs the compiler
to try and retrieve the first value for each field in any possible
value in the enum. This requires that all fields in *all variants*
implement `EnumState`, unless specifically indicated otherwise.

When this token is placed at the variant level, it overrides whichever
attribute was declared at the top level (excluding `default` and any
attribute specified by another macro).

### `last`

This attribute is equivalent to `first`, except it retrieves the last
value in the enum instead of the first.

### `auto`

This is another variant of `first` and `last` which informs the compiler
to try and use whichever value is specified as the default for any given
field in the enum. If anywhere no value is specified as the default value,
it will instead use the first value in the enum.
