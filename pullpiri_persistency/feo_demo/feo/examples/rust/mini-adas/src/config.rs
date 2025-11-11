/********************************************************************************
 * Copyright (c) 2025 Contributors to the Eclipse Foundation
 *
 * See the NOTICE file(s) distributed with this work for additional
 * information regarding copyright ownership.
 *
 * This program and the accompanying materials are made available under the
 * terms of the Apache License Version 2.0 which is available at
 * https://www.apache.org/licenses/LICENSE-2.0
 *
 * SPDX-License-Identifier: Apache-2.0
 ********************************************************************************/

use crate::activities::components::{
    Camera, EnvironmentRenderer, NeuralNet, Radar,
    SteeringController, CarModeCalculator, CarDataPublisher, AutonomousModePublisher, 
    ManualModePublisher, EmergencyModePublisher,
};
use crate::activities::messages::{BrakeInstruction, CameraImage, RadarScan, Scene, Steering, CarData, AutonomousCarData, ManualCarData, EmergencyModeData};
use crate::ffi::{lane_assist, trajectory_visualizer};
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use feo::activity::{ActivityBuilder, ActivityIdAndBuilder};
use feo::ids::{ActivityId, AgentId, WorkerId};
use feo::topicspec::{Direction, TopicSpecification};
use feo_com::interface::ComBackend;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub type WorkerAssignment = (WorkerId, Vec<(ActivityId, Box<dyn ActivityBuilder>)>);

// For each activity, list the activities it needs to wait for
pub type ActivityDependencies = HashMap<ActivityId, Vec<ActivityId>>;

#[cfg(feature = "com_iox2")]
pub const COM_BACKEND: ComBackend = ComBackend::Iox2;
#[cfg(feature = "com_linux_shm")]
pub const COM_BACKEND: ComBackend = ComBackend::LinuxShm;

pub const BIND_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8081);
pub const BIND_ADDR2: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8082);

pub const TOPIC_INFERRED_SCENE: &str = "feo/com/vehicle/inferred/scene";
pub const TOPIC_CONTROL_BRAKES: &str = "feo/com/vehicle/control/brakes";
pub const TOPIC_CONTROL_STEERING: &str = "feo/com/vehicle/control/steering";
pub const TOPIC_CAMERA_FRONT: &str = "feo/com/vehicle/camera/front";
pub const TOPIC_RADAR_FRONT: &str = "feo/com/vehicle/radar/front";
pub const TOPIC_CAR_DATA: &str = "feo/com/vehicle/car_data";
pub const TOPIC_AUTONOMOUS_DATA: &str = "feo/com/vehicle/autonomous_data";
pub const TOPIC_MANUAL_DATA: &str = "feo/com/vehicle/manual_data";
pub const TOPIC_EMERGENCY_DATA: &str = "feo/com/vehicle/emergency_data";

/// Allow up to two recorder processes (that potentially need to subscribe to every topic)
pub const MAX_ADDITIONAL_SUBSCRIBERS: usize = 2;

pub fn socket_paths() -> (PathBuf, PathBuf) {
    (
        Path::new("/tmp/feo_listener1.socket").to_owned(),
        Path::new("/tmp/feo_listener2.socket").to_owned(),
    )
}

pub fn agent_assignments() -> HashMap<AgentId, Vec<(WorkerId, Vec<ActivityIdAndBuilder>)>> {
    // Assign activities to different workers
    let w40: WorkerAssignment = (
        40.into(),
        vec![(
            0.into(),
            Box::new(|id| Camera::build(id, TOPIC_CAMERA_FRONT)),
        )],
    );
    let w41: WorkerAssignment = (
        41.into(),
        vec![(1.into(), Box::new(|id| Radar::build(id, TOPIC_RADAR_FRONT)))],
    );

    let w42: WorkerAssignment = (
        42.into(),
        vec![
            (
                2.into(),
                Box::new(|id| {
                    NeuralNet::build(
                        id,
                        TOPIC_CAMERA_FRONT,
                        TOPIC_RADAR_FRONT,
                        TOPIC_INFERRED_SCENE,
                    )
                }),
            ),
            (
                3.into(),
                Box::new(|id| EnvironmentRenderer::build(id, TOPIC_INFERRED_SCENE)),
            ),
            (
                9.into(),
                Box::new(|id| CarModeCalculator::build(id, TOPIC_INFERRED_SCENE, TOPIC_CAR_DATA)),
            ),
            (
                10.into(),
                Box::new(|id| CarDataPublisher::build(id, TOPIC_CAR_DATA)),
            ),
            (
                11.into(),
                Box::new(|id| AutonomousModePublisher::build(id, TOPIC_CAR_DATA, TOPIC_INFERRED_SCENE, TOPIC_AUTONOMOUS_DATA)),
            ),
            (
                12.into(),
                Box::new(|id| ManualModePublisher::build(id, TOPIC_CAR_DATA, TOPIC_INFERRED_SCENE, TOPIC_MANUAL_DATA)),
            ),
            (
                13.into(),
                Box::new(|id| EmergencyModePublisher::build(id, TOPIC_CAR_DATA, TOPIC_INFERRED_SCENE, TOPIC_EMERGENCY_DATA)),
            ),
        ],
    );

    let w44: WorkerAssignment = (
        44.into(),
        vec![
            (5.into(), Box::new(|id| lane_assist::CppActivity::build(id))),
            (
                7.into(),
                Box::new(|id| SteeringController::build(id, TOPIC_CONTROL_STEERING)),
            ),
            (
                8.into(),
                Box::new(|id| trajectory_visualizer::CppActivity::build(id)),
            ),
        ],
    );

    // Assign workers to pools with exactly one pool belonging to one agent
    #[cfg(any(
        feature = "signalling_direct_tcp",
        feature = "signalling_direct_unix",
        feature = "signalling_relayed_tcp",
        feature = "signalling_relayed_unix"
    ))]
    let assignment = [
        (100.into(), vec![w40, w41]),
        (101.into(), vec![w42]),
        (102.into(), vec![w44]),
    ]
    .into_iter()
    .collect();

    #[cfg(feature = "signalling_direct_mpsc")]
    let assignment = [(100.into(), vec![w40, w41, w42, w44])]
        .into_iter()
        .collect();

    assignment
}

pub fn activity_dependencies() -> ActivityDependencies {
    let dependencies = [
        // Camera
        (0.into(), vec![]),
        // Radar
        (1.into(), vec![]),
        // NeuralNet
        (2.into(), vec![0.into(), 1.into()]),
        // EnvironmentRenderer
        (3.into(), vec![2.into()]),
        // LaneAssist
        (5.into(), vec![2.into()]),
        // SteeringController
        (7.into(), vec![5.into()]),
        // TrajectoryVisualizer
        (8.into(), vec![5.into()]),
        // CarModeCalculator
        (9.into(), vec![2.into()]),
        // CarDataPublisher
        (10.into(), vec![9.into()]),
        // AutonomousModePublisher
        (11.into(), vec![10.into()]),
        // ManualModePublisher
        (12.into(), vec![10.into()]),
        // EmergencyModePublisher
        (13.into(), vec![10.into()]),
    ];

    dependencies.into()
}

pub fn topic_dependencies<'a>() -> Vec<TopicSpecification<'a>> {
    use Direction::*;

    vec![
        TopicSpecification::new::<CameraImage>(
            TOPIC_CAMERA_FRONT,
            vec![(0.into(), Outgoing), (2.into(), Incoming)],
        ),
        TopicSpecification::new::<RadarScan>(
            TOPIC_RADAR_FRONT,
            vec![(1.into(), Outgoing), (2.into(), Incoming)],
        ),
        TopicSpecification::new::<Scene>(
            TOPIC_INFERRED_SCENE,
            vec![
                (2.into(), Outgoing),
                (3.into(), Incoming),
                (5.into(), Incoming),
                (9.into(), Incoming),
                (11.into(), Incoming),
                (12.into(), Incoming),
                (13.into(), Incoming),
            ],
        ),
        TopicSpecification::new::<Steering>(
            TOPIC_CONTROL_STEERING,
            vec![(5.into(), Outgoing), (7.into(), Incoming)],
        ),
        TopicSpecification::new::<CarData>(
            TOPIC_CAR_DATA,
            vec![
                (9.into(), Outgoing),
                (10.into(), Incoming),
                (11.into(), Incoming),
                (12.into(), Incoming),
                (13.into(), Incoming),
            ],
        ),
        TopicSpecification::new::<AutonomousCarData>(
            TOPIC_AUTONOMOUS_DATA,
            vec![(11.into(), Outgoing)],
        ),
        TopicSpecification::new::<ManualCarData>(
            TOPIC_MANUAL_DATA,
            vec![(12.into(), Outgoing)],
        ),
        TopicSpecification::new::<EmergencyModeData>(
            TOPIC_EMERGENCY_DATA,
            vec![(13.into(), Outgoing)],
        ),
    ]
}

pub fn worker_agent_map() -> HashMap<WorkerId, AgentId> {
    agent_assignments()
        .iter()
        .flat_map(|(aid, w)| w.iter().map(move |(wid, _)| (*wid, *aid)))
        .collect()
}

pub fn agent_assignments_ids() -> HashMap<AgentId, Vec<(WorkerId, Vec<ActivityId>)>> {
    agent_assignments()
        .into_iter()
        .map(|(k, v)| {
            (
                k,
                v.into_iter()
                    .map(|(w, a)| (w, a.into_iter().map(|(a, _)| a).collect()))
                    .collect(),
            )
        })
        .collect()
}
