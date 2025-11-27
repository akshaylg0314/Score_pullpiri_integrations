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
    SteeringController, VehiclePublisher, EmergencyBraking, BrakeController,
    DriverMonitoringSystem, LaneKeepAssistSystem, DistractionPublisher,
};
use crate::activities::messages::{
    BrakeInstruction, CameraImage, DmsState,
    LaneChangeCommand, LkasState, RadarScan, Scene, Steering, VehicleData
};
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
pub const TOPIC_CONTROL_STEERING: &str = "feo/com/vehicle/control/steering";
pub const TOPIC_CONTROL_BRAKES: &str = "feo/com/vehicle/control/brakes";
pub const TOPIC_CAMERA_FRONT: &str = "feo/com/vehicle/camera/front";
pub const TOPIC_RADAR_FRONT: &str = "feo/com/vehicle/radar/front";
pub const TOPIC_VEHICLE_DATA: &str = "feo/com/vehicle/data/published";
pub const TOPIC_DMS_DATA: &str = "feo/com/vehicle/dms/data";
pub const TOPIC_LKAS_DATA: &str = "feo/com/vehicle/lkas/data";
pub const TOPIC_LANE_CHANGE: &str = "feo/com/vehicle/lkas/lane_change";

/// Storage directory for ADAS persistency data
pub const ADAS_DATA_DIR: &str = "./adas_data";

/// Allow up to two recorder processes (that potentially need to subscribe to every topic)
pub const MAX_ADDITIONAL_SUBSCRIBERS: usize = 0;

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
            // Activity 11: Driver Monitoring System
            (
                11.into(),
                Box::new(|id| DriverMonitoringSystem::build(id, TOPIC_DMS_DATA)),
            ),
        ],
    );

    let w43: WorkerAssignment = (
        43.into(),
        vec![
            (
                6.into(),
                Box::new(|id| EmergencyBraking::build(id, TOPIC_INFERRED_SCENE, TOPIC_CONTROL_BRAKES)),
            ),
            (5.into(), Box::new(|id| lane_assist::CppActivity::build(id))),
            (
                7.into(),
                Box::new(|id| SteeringController::build(id, TOPIC_CONTROL_STEERING)),
            ),
            (
                10.into(),
                Box::new(|id| BrakeController::build(id, TOPIC_CONTROL_BRAKES)),
            ),
        ],
    );
    let w44: WorkerAssignment = (
        44.into(),
        vec![
            (
                8.into(),
                Box::new(|id| trajectory_visualizer::CppActivity::build(id)),
            ),
            // Activity 12: Lane Keep Assistance System
            (
                12.into(),
                Box::new(|id| LaneKeepAssistSystem::build(
                    id,
                    TOPIC_INFERRED_SCENE,
                    TOPIC_DMS_DATA,
                    TOPIC_LKAS_DATA,
                    TOPIC_LANE_CHANGE
                )),
            ),
            // Activity 13: Distraction Publisher (DDS)
            (
                13.into(),
                Box::new(|id| DistractionPublisher::build(id, TOPIC_DMS_DATA)),
            ),
            // Activity 9: Vehicle Publisher (modified to use DashboardData)
            (
                9.into(),
                Box::new(|id| VehiclePublisher::build(
                    id, 
                    TOPIC_INFERRED_SCENE, 
                    TOPIC_CONTROL_STEERING,
                    TOPIC_DMS_DATA,
                    TOPIC_LKAS_DATA
                )),
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
        (102.into(), vec![w43, w44]),
    ]
    .into_iter()
    .collect();

    #[cfg(feature = "signalling_direct_mpsc")]
    let assignment = [(100.into(), vec![w40, w41, w42, w43, w44])]
        .into_iter()
        .collect();

    assignment
}

pub fn activity_dependencies() -> ActivityDependencies {
    //      Primary              |       Secondary1         |                  Secondary2
    // ---------------------------------------------------------------------------------------------------
    //
    //   Camera(40)   Radar(41)
    //        \           \
    //                                 NeuralNet(42)
    //                                      |                           \                     \
    //                             EnvironmentRenderer(42)       EmergencyBraking(43)    LaneAssist(44)
    //                                                                   |                     |
    //                                                            BrakeController(43)   SteeringController(44)

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
        // EmergencyBraking
        (6.into(), vec![2.into()]),
        // SteeringController
        (7.into(), vec![5.into()]),
        // TrajectoryVisualizer
        (8.into(), vec![5.into()]),
        // VehiclePublisher (depends on Scene, Steering, DMS, LKAS)
        (9.into(), vec![2.into(), 5.into(), 11.into(), 12.into()]),
        // BrakeController
        (10.into(), vec![6.into()]),
        // DMS (no dependencies - sensor simulation)
        (11.into(), vec![]),
        // LKAS (depends on Scene)
        (12.into(), vec![2.into()]),
        // DistractionPublisher (depends on DMS)
        (13.into(), vec![11.into()]),
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
                (6.into(), Incoming),
                (9.into(), Incoming),
                (12.into(), Incoming),  // LKAS also needs scene
            ],
        ),
        TopicSpecification::new::<Steering>(
            TOPIC_CONTROL_STEERING,
            vec![(5.into(), Outgoing), (7.into(), Incoming), (9.into(), Incoming)],
        ),
        TopicSpecification::new::<BrakeInstruction>(
            TOPIC_CONTROL_BRAKES,
            vec![(6.into(), Outgoing), (10.into(), Incoming)],
        ),
        TopicSpecification::new::<VehicleData>(
            TOPIC_VEHICLE_DATA,
            vec![(9.into(), Outgoing)],
        ),
        // DMS internal topic
        TopicSpecification::new::<DmsState>(
            TOPIC_DMS_DATA,
            vec![(11.into(), Outgoing), (9.into(), Incoming), (13.into(), Incoming)],
        ),
        // LKAS internal topic
        TopicSpecification::new::<LkasState>(
            TOPIC_LKAS_DATA,
            vec![(12.into(), Outgoing), (9.into(), Incoming)],
        ),
        // Lane change command topic
        TopicSpecification::new::<LaneChangeCommand>(
            TOPIC_LANE_CHANGE,
            vec![(12.into(), Outgoing), (9.into(), Incoming)],
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
