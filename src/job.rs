use nix::unistd::Pid;

pub enum JobStatus {
    Stopped,
    Foreground,
    Background,
}

pub struct Job {
    pub id: usize,
    pub pgid: Pid,
    pub job_status: JobStatus,
    pub command: Option<String>,
}
