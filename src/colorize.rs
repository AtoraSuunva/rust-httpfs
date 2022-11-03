use owo_colors::{OwoColorize, Stream, SupportsColorsDisplay};

// Shortcut for <Sized>.if_supports_color(Stream::Stdout)
pub trait MColorize: Sized {
    /// Colorize only if supports color on stdout
    #[must_use]
    fn out_color<'a, Out, ApplyFn>(
        &'a self,
        apply: ApplyFn,
    ) -> SupportsColorsDisplay<'a, Self, Out, ApplyFn>
    where
        ApplyFn: Fn(&'a Self) -> Out,
    {
        self.if_supports_color(Stream::Stdout, apply)
    }
}

impl<D: Sized> MColorize for D {}
