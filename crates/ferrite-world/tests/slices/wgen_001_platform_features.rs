use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::platform::{
    END_PLATFORM_WRITE_FLAGS, EndGatewayConfig, EndGatewayWorld, place_end_gateway,
};
use ferrite_world::id::BlockStateId;

#[test]
fn end_gateway_uses_x_fastest_matrix_and_configures_exit_immediately_after_center() {
    let origin = BlockPos::new(4, 70, -6);
    let exit = BlockPos::new(100, 50, 0);
    let config = EndGatewayConfig {
        gateway: BlockStateId::new(1),
        bedrock: BlockStateId::new(2),
        air: BlockStateId::new(3),
        exit: Some(exit),
        exact: true,
    };
    let mut world = GatewayFixture::default();
    assert!(place_end_gateway(&mut world, origin, config, |_| true).unwrap());
    assert_eq!(world.offers.len(), 45);
    assert_eq!(
        &world.offers[..3],
        [
            (BlockPos::new(3, 68, -7), config.air, 3),
            (BlockPos::new(4, 68, -7), config.air, 3),
            (BlockPos::new(5, 68, -7), config.air, 3),
        ]
    );
    assert_eq!(
        world.offers[22],
        (origin, config.gateway, END_PLATFORM_WRITE_FLAGS)
    );
    assert_eq!(world.exit_configuration, [(origin, exit, true, 23)]);
    assert_eq!(
        &world.offers[21..24],
        [
            (BlockPos::new(3, 70, -6), config.air, 3),
            (origin, config.gateway, 3),
            (BlockPos::new(5, 70, -6), config.air, 3),
        ]
    );
}

#[derive(Debug, Default)]
struct GatewayFixture {
    offers: Vec<(BlockPos, BlockStateId, u32)>,
    exit_configuration: Vec<(BlockPos, BlockPos, bool, usize)>,
}

impl EndGatewayWorld for GatewayFixture {
    fn offer_gateway_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }

    fn configure_gateway_exit(&mut self, position: BlockPos, exit: BlockPos, exact: bool) -> bool {
        self.exit_configuration
            .push((position, exit, exact, self.offers.len()));
        true
    }
}
