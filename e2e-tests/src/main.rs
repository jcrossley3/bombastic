
mod world;
mod pages;
use cucumber::{cli, World, writer, WriterExt};
use thirtyfour::{Capabilities, WebDriver, DesiredCapabilities};
use world::{E2EContext, E2EWorld};
use futures::FutureExt;
use std::fs::File;
use std::sync::Arc;
use std::process::Command;

#[derive(cli::Args, Clone)]  // re-export of `clap::Args`
pub struct E2Ecliparser{
    ///Application URL under test
    #[arg(long)]
    pub application: String,
    ///User Name to login to Application
    #[arg(long)]
    pub user_name: String,
    ///Password to login to Application
    #[arg(long)]
    pub password: String,
    ///Select (chrome/firefox) to run the Application
    #[arg(long)]
    pub browser: String,
    /////Select (chrome/firefox) to run the Application
    //#[arg(long)]
    //pub browsercap: String,
}

#[tokio::main]
async fn main() {
    let junit_output_file = File::create(format!("{}/junit.xml", env!("CARGO_MANIFEST_DIR"))).expect("Error file creation");
    let json_output_file = File::create(format!("{}/cucumber.json", env!("CARGO_MANIFEST_DIR"))).expect("Error file creation");
    let opts = cli::Opts::<_, _, _, E2Ecliparser>::parsed();
    let _option = opts.custom.clone();
    let opts_custom = Arc::new(_option);
    E2EWorld::cucumber()
        .before(move |_, _, _, world| {
                let opts_custom = Arc::clone(&opts_custom);
                let mut context = E2EContext::new();
                Box::pin(async move {
                    let mut serverurl = "http://localhost:9515";
                    let mut caps: Capabilities = DesiredCapabilities::chrome().into();
                    if opts_custom.browser.as_str() == "firefox"{
                        caps = DesiredCapabilities::firefox().into();
                        serverurl = "http://localhost:4444";
                        Command::new("geckodriver")
                                .args(["--port=4444"])
                                .spawn()
                                .expect("Failed to execute process");
                    }else{
                        Command::new("chromedriver")
                                .args(["--port=9515"])
                                .spawn()
                                .expect("Failed to execute process");
                    };
                    let driver = WebDriver::new(serverurl, caps).await.expect("Error while creating Webdriver");
                    context.insert(driver);
                    world.context = Arc::new(context);
                    world.application = Some(opts_custom.application.clone());
                    world.browser = Some(opts_custom.browser.clone());
                    world.user_name = Some(opts_custom.user_name.clone());
                    world.password = Some(opts_custom.password.clone());
                })
            }
        .boxed_local())
        .with_cli(opts)
        .after(move |_, _, _, _, world| {
            async move {
                if let Some(world) = world {
                    let web_driver: &WebDriver = world.context.get_driver().unwrap();
                    //The WebDriver is defined using Arc insider E2Eworld and quit will kill all the earlier references
                    web_driver.clone().quit().await.unwrap();
                }
            }
            .boxed_local()
        }) 
        .with_writer(
            writer::Libtest::or_basic()
                .tee::<E2EWorld, _>(writer::JUnit::for_tee(
                    junit_output_file,
                    writer::Verbosity::default(),
                ))
                .tee::<E2EWorld, _>(writer::Json::for_tee(json_output_file))
                .normalized()
        )
        .run("tests/features/")
        .await;
}