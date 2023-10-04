
use std::time::Duration;
use crate::recovery::Acked;



// to be initialised from environment variables later
const PREVIOUS_RTT: Duration = Duration::from_millis(600);
const JUMP_WINDOW: usize = 2000;

#[derive(Debug)]
pub enum CrState {
    OBSERVE,
    RECON,
    UNVAL,
    VALIDATE,
//  RETREAT,
    NORMAL,
}

impl Default for CrState {
    fn default() -> Self { CrState::OBSERVE }
}

#[derive(Default)]
pub struct Resume {
    enabled: bool,

    cr_state: CrState,

    previous_rtt: Duration,

    jump_window: usize,

    cr_mark: u64,
}

impl std::fmt::Debug for Resume {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "cr_state={:?} ", self.cr_state)?;
        write!(f, "last_rtt={:?} ", self.previous_rtt)?;
        write!(f, "jump_window={:?} ", self.jump_window)?;
        write!(f, "cr_mark={:?} ", self.cr_mark)?;
        Ok(())
    }
}

impl Resume {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,

            //Starting at recon as draft does not yet discuss observe
            cr_state: CrState::RECON,

            previous_rtt: PREVIOUS_RTT,

            jump_window: JUMP_WINDOW,

            ..Default::default()
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new(self.enabled);
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    //this is a function to set the CR mark
    pub fn set_cr_mark(&mut self, window_high_end: u64) {
        self.cr_mark = window_high_end;
    }

    pub fn process_ack(&mut self, rtt_sample: Duration, cwnd: usize,  packet: &Acked )-> usize {
    if let CrState::RECON = self.cr_state{
        if cwnd >= self.jump_window {
            self.cr_state = CrState::NORMAL;
        }
        if rtt_sample <= self.previous_rtt / 2 || rtt_sample >= self.previous_rtt * 10 {
            self.cr_state = CrState::NORMAL;
        }
    }

     match (&self.cr_state, packet.pkt_num >= self.cr_mark) {
     (CrState::UNVAL, true) => {
        // move to validating
        self.cr_state = CrState::VALIDATE;
        // we return the jump window, CC code handles the increase in cwnd
        // and setting the CR mark
        return self.jump_window;
     }
     (CrState::VALIDATE, true) => {
     self.cr_state = CrState::NORMAL;}
     _ => {
         //in here we can handle other cases
        }
     }

    //otherwise we return 0 aka we don't touch the cwnd
    return 0;
    }

    pub fn send_packet(&mut self, flightsize: usize, cwnd: usize) -> usize {
      match (&self.cr_state, flightsize >= cwnd) {
      (CrState::RECON, true) => {
         // move to validating and update mark
        self.cr_state = CrState::UNVAL;
        // we return the jump window, CC code handles the increase in cwnd
        // and setting the CR mark
        return self.jump_window;
     }
     _ => {
        // Otherwise we don't touch the cwnd
        return 0;
        }
     }
   }
   }