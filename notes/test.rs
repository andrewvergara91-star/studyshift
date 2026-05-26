// test.rs
#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _},
    Address, Env, Symbol,
};

use crate::StudyShift;

#[test]
fn test_happy_flow() {
    let env = Env::default();
    let contract = env.register(StudyShift, ());

    let employer = Address::generate(&env);
    let student = Address::generate(&env);

    let job_id = StudyShiftClient::new(&env, &contract)
        .create_job(&employer, &Symbol::new(&env, "Design"), &100);

    let client = StudyShiftClient::new(&env, &contract);

    client.apply(&student, &job_id);
    client.assign(&employer, &job_id, &student);
    client.submit(&student, &job_id);

    // funding + payment steps assumed in integration
}

#[test]
#[should_panic]
fn test_unauthorized_assign() {
    let env = Env::default();
    let contract = env.register(StudyShift, ());

    let employer = Address::generate(&env);
    let attacker = Address::generate(&env);
    let student = Address::generate(&env);

    let client = StudyShiftClient::new(&env, &contract);

    let job_id = client.create_job(&employer, &Symbol::new(&env, "Task"), &50);

    client.assign(&attacker, &job_id, &student);
}

#[test]
#[should_panic]
fn test_duplicate_assignment_state() {
    let env = Env::default();
    let contract = env.register(StudyShift, ());

    let employer = Address::generate(&env);
    let student = Address::generate(&env);

    let client = StudyShiftClient::new(&env, &contract);

    let job_id = client.create_job(&employer, &Symbol::new(&env, "Task"), &50);

    client.assign(&employer, &job_id, &student);
    client.assign(&employer, &job_id, &student); // invalid state
}

#[test]
fn test_storage() {
    let env = Env::default();
    let contract = env.register(StudyShift, ());

    let employer = Address::generate(&env);

    let client = StudyShiftClient::new(&env, &contract);

    let job_id = client.create_job(&employer, &Symbol::new(&env, "Test"), &99);

    let job = client.get_job(&job_id);

    assert_eq!(job.payment, 99);
}

#[test]
fn test_application_flow() {
    let env = Env::default();
    let contract = env.register(StudyShift, ());

    let employer = Address::generate(&env);
    let student = Address::generate(&env);

    let client = StudyShiftClient::new(&env, &contract);

    let job_id = client.create_job(&employer, &Symbol::new(&env, "Task"), &100);

    client.apply(&student, &job_id);

    let job = client.get_job(&job_id);

    assert!(job.applicants.len() > 0);
}