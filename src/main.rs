pub mod pp7;

use config::Config;
use openrgb::data::{Controller, DeviceType};
use openrgb::OpenRGB;
use restson::RestClient;
use std::error::Error;
use std::fs::File;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time;
use tokio_retry::strategy::FixedInterval;
use tokio_retry::Retry;

/// Read the HotKey Mapping of PP7 from predefined file
/// - the file need to be in the run folder
fn read_keybindings(settings: &Config) -> Result<Vec<pp7::PP7KeyBind>, Box<dyn Error>> {
    let file = File::open(settings.get_string("BindingFile")?)?;
    let mut binds: Vec<pp7::PP7KeyBind> = Vec::new();
    let mut rdr = csv::Reader::from_reader(file);
    for result in rdr.deserialize() {
        let record: pp7::PP7KeyBind = result?;
        println!("{:?}", record);
        binds.push(record);
    }
    Ok(binds)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let settings: Config = Config::builder()
        .add_source(config::File::with_name("settings"))
        .build()?;

    // connect to local server OpenRGB service
    // and wait for it to be running
    let client = Retry::spawn(FixedInterval::from_millis(1000), || {
        println!("connecting to openrgb");
        OpenRGB::connect_to(("localhost", 6742))
    })
    .await?;

    // test the connection
    client.set_name("orgb pp7 connection").await?;
    println!("OpenRGB protocol {}", client.get_protocol_version());

    // loop over available controllers
    let count = client.get_controller_count().await?;
    for i in 0..count {
        let controller = client.get_controller(i).await?;
        println!("{:?}", controller);

        if controller.r#type == DeviceType::Keyboard {
            // Run the keyboard task if we find a openRGB keyboard
            keyboard_ctl_task(client, controller, i, &settings).await?;
            break;
        }
    }

    Ok(())
}

///
/// ## Task that controls the keyboard leds
/// **will not return - endless loop**
///
/// - request settings & updates from PP7
/// - updates the keyboard led's using the PP7 information
///
///
/// - TODO: static color keys defined by the user
/// - TODO: uses hardcoded config -> change to some config file or args (see:https://docs.rs/config/latest/config/)
///
async fn keyboard_ctl_task(
    orgb: OpenRGB<TcpStream>,
    key: Controller,
    cid: u32,
    setting: &Config,
) -> Result<(), Box<dyn Error>> {
    // parse settings for this task
    let pp7api_base_url = format!(
        "http://{host}:{port}",
        host = setting
            .get_string("Host")
            .unwrap_or("localhost".to_string()),
        port = setting.get_int("Port").unwrap_or(50001),
    );
    let led_color_enable = setting.get_bool("LedColors").unwrap_or(false);
    let led_on = setting.get_int("LedOn").unwrap_or(255);
    let led_dim = setting.get_int("LedDim").unwrap_or(5);
    let delay_interval_ms = 1000 / setting.get_int("UpdateFrequency").unwrap_or(100);

    // create PP7 api connection
    let pp7api_client = RestClient::new(&pp7api_base_url).unwrap();
    // here the first request is made, so we wait for PP to be started
    let groups = Retry::spawn(FixedInterval::from_millis(1000), || {
        println!("requesting config from proPresenter");
        pp7api_client.get::<_, pp7::GlobalGroupList>(())
    })
    .await?
    .into_inner();
    println!("{:?}", groups);

    // read keybindings file (map GroupName -> Key(char) )
    let binds = read_keybindings(setting)?;
    let mut last_uuid: String = "".to_string();

    loop {
        let bind = binds.clone();

        if let Ok(req) = pp7api_client.get::<_, pp7::PresentationRequest>(()).await {
            // request current presentation
            let pre: pp7::PresentationRequest = req.into_inner();
            let pre_groups = pre.presentation.groups;

            // only update if presentation has changed
            if pre.presentation.id.uuid != last_uuid {
                last_uuid = pre.presentation.id.uuid.clone();
                println!("{:?}", last_uuid);

                // get the group-names of the current presentation
                let groups: Vec<String> = pre_groups.iter().map(|x| x.name.clone()).collect();
                // using this group names
                // make a list of all used key bindings by this presentation
                let keys: Vec<pp7::PP7KeyBind> = bind
                    .into_iter()
                    .filter(|x| groups.contains(&x.bind))
                    .collect();

                // make a list of all LED colors of to update
                let colors: Vec<openrgb::data::Color> = key
                    .leds
                    .iter()
                    .map(|led_data| led_data.name.replace("Key: ", ""))
                    .enumerate()
                    .map(|(i, key_name)| {
                        // Find if a Key on the Keyboard is used by an active Group
                        // Find by KeyName or by LED Index
                        let matches: Vec<&pp7::PP7KeyBind> = keys
                            .iter()
                            .filter(|group_bind| group_bind.key == key_name || group_bind.num == i)
                            .collect();

                        if !matches.is_empty() {
                            if led_color_enable {
                                // get group names of the matches
                                let match_names: Vec<&String> = matches
                                    .iter()
                                    .map(|matched_bind| &matched_bind.bind)
                                    .collect();

                                // get the group matching group color
                                let g = pre_groups
                                    .iter()
                                    .find(|group| match_names.contains(&&group.name))
                                    .unwrap();
                                let c = &g.color;
                                openrgb::data::Color::new(
                                    (c.red * 255.0) as u8,
                                    (c.green * 255.0) as u8,
                                    (c.blue * 255.0) as u8,
                                )
                            } else {
                                openrgb::data::Color::new(led_on as u8, led_on as u8, led_on as u8)
                            }
                        } else {
                            openrgb::data::Color::new(led_dim as u8, led_dim as u8, led_dim as u8)
                        }
                    })
                    .collect();

                orgb.update_leds(cid, colors).await?;
            }
        }

        //TODO : use interval for constant updates
        //https://stackoverflow.com/questions/66863385/how-can-i-use-tokio-to-trigger-a-function-every-period-or-interval-in-seconds
        time::sleep(Duration::from_millis(delay_interval_ms as u64)).await;
    }
}

/*
///Used to iterate over each led and find its index
async fn keyboard_test_task(
    orgb: OpenRGB<TcpStream>,
    key: Controller,
    cid: u32,
    setting: &Config,
) -> Result<(), Box<dyn Error>> {
    let led_on = setting.get_int("LedOn").unwrap_or(255);
    let led_dim = setting.get_int("LedDim").unwrap_or(5);
    let delay_interval_ms = 1000 / setting.get_int("UpdateFrequency").unwrap_or(100);

    let mut counter: usize = 0;

    loop {
        println!("{}", counter);
        // make a list of all LED colors of to update
        let colors: Vec<openrgb::data::Color> = key
            .leds
            .iter()
            .enumerate()
            .map(|(i, _led)| {
                // Find if an Key on the Keyboard is used by an active Group
                if i == counter {
                    openrgb::data::Color::new(led_on as u8, led_on as u8, led_on as u8)
                } else {
                    openrgb::data::Color::new(led_dim as u8, led_dim as u8, led_dim as u8)
                }
            })
            .collect();

        orgb.update_leds(cid, colors).await?;

        counter += 1;

        let mut buffer = String::new();
        let stdin = std::io::stdin(); // We get `Stdin` here.
        stdin.read_line(&mut buffer)?;

        //TODO : use interval for constant updates
        //https://stackoverflow.com/questions/66863385/how-can-i-use-tokio-to-trigger-a-function-every-period-or-interval-in-seconds
        time::sleep(Duration::from_millis(delay_interval_ms as u64)).await;
    }
}
*/
