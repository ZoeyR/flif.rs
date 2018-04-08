use components::transformations::ColorRange;
use numbers::rac::RacRead;
use components::transformations::Transform;
use error::*;
use numbers::chances::{ChanceTable, UpdateTable};
use numbers::near_zero::NearZeroCoder;
use colors::{Channel, ColorSpace, ColorValue, Pixel};
const MAX_PALETTE_SIZE: usize = 30000;

pub struct Palette {
    palette: Vec<ColorValue>,
}

impl Palette {
    pub fn new<R: RacRead, T: Transform>(
        rac: &mut R,
        transformation: T,
        channels: ColorSpace,
        update_table: &UpdateTable,
    ) -> Result<Palette> {
        let mut context_a = ChanceTable::new(update_table);
        let mut context_y = ChanceTable::new(update_table);
        let mut context_i = ChanceTable::new(update_table);
        let mut context_q = ChanceTable::new(update_table);

        let size = rac.read_near_zero(0, MAX_PALETTE_SIZE, &mut context_a)?;

        unimplemented!()
    }
}

impl Transform for Palette {
    fn undo(&self, pixel: &mut Pixel) {
        unimplemented!()
    }

    fn range(&self, channel: Channel) -> ColorRange {
        unimplemented!()
    }

    fn crange(&self, channel: Channel, values: &Pixel) -> ColorRange {
        unimplemented!()
    }
}
