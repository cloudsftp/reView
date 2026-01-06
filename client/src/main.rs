use anyhow::{Context, Error};
use async_ssh2_tokio::{AuthMethod, Client, ServerCheckMethod};
use log::{debug, error, info};
use tokio::{sync::mpsc::{self, Receiver}};

const HEIGHT: usize = 1872;
const WIDTH: usize = 1404;
const PIXEL_FORMAT: &str = "bgra";
const BYTES_PER_PIXEL: usize = 4;
const FILE: &str = ":mem:";
const SKIP_OFFSET: usize = 2629636;

const PORT: usize = 6680;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Debug)
        .parse_default_env()
        .init();

    //run_restream().await?;

    let client = connect_ssh().await?;
    let (restream_command_future, mut restream_command_stdout) = start_server(&client).await?;
    tokio::pin!(restream_command_future);
    loop {
        tokio::select! {
            restream_exit_code = &mut restream_command_future => {
                error!("restream command exited with code {}", restream_exit_code.context("could not execute restream command")?);

                let restream_output = receive_output(&mut restream_command_stdout).await?;
                error!("stdout+stderr: (next line)\n\n{}\n", restream_output);
            },
        }
    }

    Ok(())
}

async fn receive_output(stdout: &mut Receiver<Vec<u8>>) -> Result<String, Error> {
    let mut buf = vec![];
    while let Some(mut data) = stdout.recv().await {
        buf.append(&mut data);
    }
    return Ok(String::from_utf8_lossy(&buf).to_string());
}

async fn connect_ssh() -> Result<Client, Error> {
    info!("connecting to reMarkable");
    Client::connect(
        ("192.168.2.118", 22),
        "root",
        AuthMethod::PrivateKeyFile {
            key_file_path: "/home/fabi/.ssh/id_ed25519".into(),
            key_pass: None,
        },
        ServerCheckMethod::NoCheck,
    )
    .await
    .context("could not connect to reMarkable tablet")
}

async fn start_server(client: &Client) -> Result<(impl Future<Output = Result<u32, async_ssh2_tokio::Error>>, Receiver<Vec<u8>>), Error> {
    let (stdout_tx, stdout_rx) = mpsc::channel(10);

    let restream_command = Box::leak(Box::new(format!(
        "./restream --height {} --width {} --bytes-per-pixel {} --file {} --skip {} --listen {}",
        HEIGHT, WIDTH, BYTES_PER_PIXEL, FILE, SKIP_OFFSET, PORT,
    )));

    debug!("spawning restream");
    let exec_future = client.execute_io(restream_command, stdout_tx, None, None, false, None);

    Ok((exec_future, stdout_rx))
}

/*
async fn run_restream() -> Result<(), Error> {
    info!("connecting to reMarkable");
    let client = Client::connect(
        ("192.168.2.118", 22),
        "root",
        AuthMethod::PrivateKeyFile {
            key_file_path: "/home/fabi/.ssh/id_ed25519".into(),
            key_pass: None,
        },
        ServerCheckMethod::NoCheck,
    )
    .await
    .context("could not connect to reMarkable tablet")?;

    let (stdout_tx, mut stdout_rx) = mpsc::channel(10);

    let mut command = Command::new("ffplay");
    command.args(&[
        "-vcodec",
        "rawvideo",
        "-f",
        "rawvideo",
        "-pixel_format",
        PIXEL_FORMAT,
        "-video_size",
        &format!("{},{}", WIDTH, HEIGHT),
        "-i",
        "-",
    ]);
    command.stdin(Stdio::piped());
    debug!("spawning ffmpeg");
    let mut command = command.spawn().context("could not spawn ffmpeg command")?;
    debug!("getting stdin");
    let mut stdin = command
        .stdin
        .take()
        .context("could not get stdin of ffmpeg")?;

    debug!("spawning restream thread");
    //let thread = thread::spawn(async move || {
    let restream_command = format!(
        "./restream --height {} --width {} --bytes-per-pixel {} --file {} --skip {}",
        HEIGHT, WIDTH, BYTES_PER_PIXEL, FILE, SKIP_OFFSET,
    );
    debug!("spawning restream");
    let exec_future = client.execute_io(&restream_command, stdout_tx, None, None, false, None);
    debug!("pinning future");
    tokio::pin!(exec_future);
    loop {
        debug!("selecting");
        tokio::select! {
            result = &mut exec_future => break,
            Some(stdout) = stdout_rx.recv() => {
                //debug!("ssh stdout: {}", String::from_utf8_lossy(&stdout));
                debug!("read some bytes (length: {})", stdout.len());
                stdin.write(&stdout).await.unwrap();
            },
        };
    }
    //});

    command
        .wait()
        .await
        .context("could not wait for command to finish")?;

    /*
    debug!("command exited with error code {}", result);
    debug!("sdtout: {:?}", result_stdout.iter().map(|line| String::from_utf8_lossy(&line)).collect_vec());
    */

    //client.disconnect().await.context("could not disconnect from reMarkable")?;

    Ok(())
}
*/
