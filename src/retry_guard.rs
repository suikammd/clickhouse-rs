use std::{
    mem,
    time::Duration,
};

use tokio::timer::Interval;

use log::warn;

use crate::{Client, ClientHandle, errors::Result, types::OptionsSource};

pub(crate) async fn retry_guard(
    handle: &mut ClientHandle,
    source: &OptionsSource,
    max_attempt: usize,
    duration: Duration
) -> Result<()> {
    let mut attempt = 0;
    let mut skip_check = false;

    loop {
        if skip_check {
            skip_check = false;
        } else {
            match check(handle).await {
                Ok(()) => return Ok(()),
                Err(err) => {
                    if attempt >= max_attempt {
                        return Err(err);
                    }
                },
            }
        }

        match reconnect(handle, source).await {
            Ok(()) => continue,
            Err(err) => {
                skip_check = true;
                if attempt >= max_attempt {
                    return Err(err);
                }

                let mut interval = Interval::new_interval(duration);
                interval.next().await;
            }
        }

        attempt += 1;
    }
}

async fn check(c: &mut ClientHandle) -> Result<()> {
    c.ping().await
}

async fn reconnect(c: &mut ClientHandle, source: &OptionsSource) -> Result<()> {
    warn!("[reconnect]");
    let mut nc = Client::open(&source).await?;
    mem::swap(c, &mut nc);
    Ok(())
}