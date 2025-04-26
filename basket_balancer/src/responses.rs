use basket_communication::external;

#[inline(always)]
pub(crate) async fn send_hap_response(response: external::QueuePositionUpdate) {
    println!("HAP | sending response: {:?}", response);
}
