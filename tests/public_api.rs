use lotusx::core::kernel::ReqwestRest;

fn assert_connector_type<T>() {}

#[test]
fn root_reexports_active_connector_types() {
    assert_connector_type::<lotusx::BackpackConnector<ReqwestRest>>();
    assert_connector_type::<lotusx::BinanceConnector<ReqwestRest>>();
    assert_connector_type::<lotusx::BinancePerpConnector<ReqwestRest>>();
    assert_connector_type::<lotusx::BybitConnector<ReqwestRest>>();
    assert_connector_type::<lotusx::BybitPerpConnector<ReqwestRest>>();
    assert_connector_type::<lotusx::HyperliquidConnector<ReqwestRest>>();
    assert_connector_type::<lotusx::OkxConnector<ReqwestRest>>();
    assert_connector_type::<lotusx::ParadexConnector<ReqwestRest>>();
}
