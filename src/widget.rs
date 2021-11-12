use tui::widgets::StatefulWidget;

use crate::Matrix;

///
pub struct MatrixWidget<F, T>
where
    F: FnMut(Option<u16>) -> T,
{
    ///
    f: F,
}

impl<F, T> std::fmt::Debug for MatrixWidget<F, T>
where
    F: FnMut(Option<u16>) -> T,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MatrixWidget { ... }")
    }
}

impl<F, T> MatrixWidget<F, T>
where
    F: FnMut(Option<u16>) -> T,
{
    ///
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F> StatefulWidget for MatrixWidget<F, tui::buffer::Cell>
where
    F: FnMut(Option<u16>) -> tui::buffer::Cell,
{
    type State = Matrix<tui::buffer::Cell>;

    fn render(
        self,
        area: tui::layout::Rect,
        buf: &mut tui::buffer::Buffer,
        state: &mut Self::State,
    ) {
        state.notify_size(area.width, area.height);
        state.tick(self.f);
        for i in 0..area.height {
            for j in 0..area.width {
                std::mem::swap(buf.get_mut(j, i), &mut state.map[i as usize][j as usize]);
            }
        }
    }
}
