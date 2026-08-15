use next_runtime::{TickReport, TransactionStage};

pub(super) fn assert_world_services_stage_order(reports: &[TickReport]) {
    let world_services_reports = reports
        .iter()
        .filter(|report| {
            report
                .stage_trace
                .iter()
                .any(|entry| entry.stage == TransactionStage::WorldStreamingCommit)
        })
        .collect::<Vec<_>>();
    assert_eq!(world_services_reports.len(), reports.len());
    for report in world_services_reports {
        let ingress = report
            .stage_trace
            .iter()
            .position(|entry| entry.stage == TransactionStage::IngressCommit)
            .expect("ingress commit stage");
        let streaming = report
            .stage_trace
            .iter()
            .position(|entry| entry.stage == TransactionStage::WorldStreamingCommit)
            .expect("world streaming stage");
        let physical = report
            .stage_trace
            .iter()
            .position(|entry| entry.stage == TransactionStage::PhysicalStep)
            .expect("physical stage");
        assert!(ingress < streaming && streaming < physical);
    }
}

pub(super) fn text_resolver(
    package: &next_project::ActivatedProjectPackage,
    locale: &str,
) -> next_presentation::TextCatalogResolverV1 {
    next_presentation::TextCatalogResolverV1::new(package.project.text_catalogs.clone(), locale)
        .expect("text resolver")
}
