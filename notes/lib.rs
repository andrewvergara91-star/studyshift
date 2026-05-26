// lib.rs
#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype,
    Address, Env, Symbol, Vec, token, log,
};

#[derive(Clone)]
#[contracttype]
pub enum JobStatus {
    Open,
    Assigned,
    InProgress,
    Submitted,
    Completed,
    Paid,
}

#[derive(Clone)]
#[contracttype]
pub struct Job {
    pub employer: Address,
    pub student: Option<Address>,
    pub title: Symbol,
    pub payment: i128,
    pub status: JobStatus,
    pub applicants: Vec<Address>,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Job(u64),
    JobCount,
    Token, // USDC contract address
}

#[contract]
pub struct StudyShift;

#[contractimpl]
impl StudyShift {

    // Set accepted token (USDC)
    pub fn set_token(env: Env, token: Address) {
        env.storage().persistent().set(&DataKey::Token, &token);
    }

    fn get_token(env: &Env) -> Address {
        env.storage().persistent().get(&DataKey::Token).unwrap()
    }

    // Create job + lock escrow
    pub fn create_job(
        env: Env,
        employer: Address,
        title: Symbol,
        payment: i128,
    ) -> u64 {

        employer.require_auth();

        let mut count: u64 =
            env.storage().persistent().get(&DataKey::JobCount).unwrap_or(0);

        count += 1;

        let job = Job {
            employer: employer.clone(),
            student: None,
            title,
            payment,
            status: JobStatus::Open,
            applicants: Vec::new(&env),
        };

        env.storage().persistent().set(&DataKey::Job(count), &job);
        env.storage().persistent().set(&DataKey::JobCount, &count);

        log!(&env, "Job created");

        count
    }

    // Employer deposits funds into escrow
    pub fn fund_job(env: Env, employer: Address, job_id: u64) {

        employer.require_auth();

        let token_client = token::Client::new(&env, &Self::get_token(&env));

        let mut job: Job =
            env.storage().persistent().get(&DataKey::Job(job_id)).unwrap();

        if job.employer != employer {
            panic!("Not employer");
        }

        // Transfer funds into contract (escrow)
        token_client.transfer(
            &employer,
            &env.current_contract_address(),
            &job.payment,
        );

        job.status = JobStatus::Assigned;

        env.storage().persistent().set(&DataKey::Job(job_id), &job);

        log!(&env, "Job funded into escrow");
    }

    // Students apply
    pub fn apply(env: Env, student: Address, job_id: u64) {

        student.require_auth();

        let mut job: Job =
            env.storage().persistent().get(&DataKey::Job(job_id)).unwrap();

        if job.status != JobStatus::Open && job.status != JobStatus::Assigned {
            panic!("Not open");
        }

        job.applicants.push_back(student.clone());

        env.storage().persistent().set(&DataKey::Job(job_id), &job);

        log!(&env, "Applied");
    }

    // Employer assigns student
    pub fn assign(env: Env, employer: Address, job_id: u64, student: Address) {

        employer.require_auth();

        let mut job: Job =
            env.storage().persistent().get(&DataKey::Job(job_id)).unwrap();

        if job.employer != employer {
            panic!("Unauthorized");
        }

        job.student = Some(student);
        job.status = JobStatus::InProgress;

        env.storage().persistent().set(&DataKey::Job(job_id), &job);

        log!(&env, "Assigned");
    }

    // Student submits work
    pub fn submit(env: Env, student: Address, job_id: u64) {

        student.require_auth();

        let mut job: Job =
            env.storage().persistent().get(&DataKey::Job(job_id)).unwrap();

        if job.student.clone().unwrap() != student {
            panic!("Not assigned student");
        }

        job.status = JobStatus::Submitted;

        env.storage().persistent().set(&DataKey::Job(job_id), &job);

        log!(&env, "Submitted");
    }

    // Employer approves + releases payment
    pub fn approve_and_pay(env: Env, employer: Address, job_id: u64) {

        employer.require_auth();

        let token_client = token::Client::new(&env, &Self::get_token(&env));

        let mut job: Job =
            env.storage().persistent().get(&DataKey::Job(job_id)).unwrap();

        if job.employer != employer {
            panic!("Unauthorized");
        }

        if job.status != JobStatus::Submitted {
            panic!("Not submitted");
        }

        let student = job.student.clone().unwrap();

        // Release escrow to student
        token_client.transfer(
            &env.current_contract_address(),
            &student,
            &job.payment,
        );

        job.status = JobStatus::Paid;

        env.storage().persistent().set(&DataKey::Job(job_id), &job);

        log!(&env, "Paid");
    }

    // View job
    pub fn get_job(env: Env, job_id: u64) -> Job {
        env.storage().persistent().get(&DataKey::Job(job_id)).unwrap()
    }
}