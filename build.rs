fn main() {
    /*let config = slint_build::CompilerConfiguration::new()
        .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer);
    slint_build::compile_with_config("ui/mainwindow.slint", config).unwrap();*/
    slint_build::compile("ui/mainwindow.slint").unwrap();
}
