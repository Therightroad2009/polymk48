// SPDX-FileCopyrightText: 2021 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::dto::*;
use crate::id::*;
use crate::name::*;
use crate::web_socket::WebSocketProtocol;
use crate::UnixTime;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU8;
use std::sync::Arc;

/// Pass the following query parameters to the system endpoint to inform server routing.
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemQuery {
    /// Express a [`ServerId`] preference. It is not guaranteed to be honored.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_id: Option<ServerId>,
    /// Express a region preference. It is not guaranteed to be honored.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region_id: Option<RegionId>,
    /// Express a preference in being placed with the inviting player. It is not guaranteed to be honored.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invitation_id: Option<InvitationId>,
}

/// Response to system request.
#[derive(Serialize, Deserialize)]
#[serde(rename = "camelCase")]
pub struct SystemResponse {
    /// The [`ServerId`] matching the invitation, or closest to the client.
    pub server_id: Option<ServerId>,
}

/// Some nonsense required by actix
/// See https://docs.rs/actix/latest/actix/dev/trait.MessageResponse.html
#[cfg(feature = "server")]
impl<A, M> actix::dev::MessageResponse<A, M> for SystemResponse
where
    A: actix::Actor,
    M: actix::Message<Result = SystemResponse>,
{
    fn handle(self, _ctx: &mut A::Context, tx: Option<actix::dev::OneshotSender<M::Result>>) {
        if let Some(tx) = tx {
            let _ = tx.send(self);
        }
    }
}

/// Response to status request.
#[derive(Serialize, Deserialize)]
pub struct StatusResponse {
    /// If false, this server cannot be relied on and should be replaced.
    pub healthy: bool,
    /// Region of this server.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region_id: Option<RegionId>,
    /// What server this server is redirecting to.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_server_id: Option<ServerId>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_hash: Option<u64>,
    /// Number of (real) players.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_count: Option<u32>,
    /// Dying servers, in need of DNS replacement, according to this server.
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dying_server_ids: Vec<ServerId>,
}

/// Some nonsense required by actix
/// See https://docs.rs/actix/latest/actix/dev/trait.MessageResponse.html
#[cfg(feature = "server")]
impl<A, M> actix::dev::MessageResponse<A, M> for StatusResponse
where
    A: actix::Actor,
    M: actix::Message<Result = StatusResponse>,
{
    fn handle(self, _ctx: &mut A::Context, tx: Option<actix::dev::OneshotSender<M::Result>>) {
        if let Some(tx) = tx {
            let _ = tx.send(self);
        }
    }
}

/// Initiate a websocket with these optional parameters in the URL query string.
#[derive(Debug, Serialize, Deserialize)]
pub struct WebSocketQuery {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<WebSocketProtocol>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arena_id: Option<ArenaId>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<SessionId>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invitation_id: Option<InvitationId>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer: Option<Referrer>,
}

/// Client to server request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Request<GR> {
    Chat(ChatRequest),
    Client(ClientRequest),
    Game(GR),
    Invitation(InvitationRequest),
    Player(PlayerRequest),
    Team(TeamRequest),
}

#[cfg(feature = "server")]
impl<GR: Serialize + serde::de::DeserializeOwned + actix::Message> actix::Message for Request<GR>
where
    <GR as actix::Message>::Result: Serialize + serde::de::DeserializeOwned,
{
    type Result = Update<GR::Result>;
}

/// Server to client update.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(actix::Message))]
#[cfg_attr(feature = "server", rtype(result = "()"))]
pub enum Update<GU> {
    Chat(ChatUpdate),
    Client(ClientUpdate),
    Game(GU),
    Invitation(InvitationUpdate),
    Leaderboard(LeaderboardUpdate),
    Liveboard(LiveboardUpdate),
    Player(PlayerUpdate),
    System(SystemUpdate),
    Team(TeamUpdate),
}

/// Admin requests are from the admin interface to the core service.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AdminRequest {
    RequestDay {
        filter: Option<MetricFilterDto>,
    },
    RequestGames,
    RequestPlayers,
    OverridePlayerAlias {
        player_id: PlayerId,
        alias: PlayerAlias,
    },
    MutePlayer {
        player_id: PlayerId,
        minutes: usize,
    },
    RestrictPlayer {
        player_id: PlayerId,
        minutes: usize,
    },
    RequestServers,
    RequestSeries {
        game_id: GameId,
        period_start: Option<UnixTime>,
        period_stop: Option<UnixTime>,
        // Resolution in hours.
        resolution: Option<NonZeroU8>,
    },
    RequestSummary {
        filter: Option<MetricFilterDto>,
    },
    RequestReferrers,
    RequestUserAgents,
    SendChat {
        // If None, goes to all players.
        player_id: Option<PlayerId>,
        alias: PlayerAlias,
        message: String,
    },
    RequestRedirect,
    SetRedirect(Option<ServerId>),
    SetProfiler(bool),
}

/// Admin related responses from the server.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AdminUpdate {
    ChatSent,
    DayRequested(Arc<[(UnixTime, MetricsDataPointDto)]>),
    GamesRequested(Box<[(GameId, f32)]>),
    PlayersRequested(Box<[AdminPlayerDto]>),
    PlayerAliasOverridden(PlayerAlias),
    PlayerMuted(usize),
    HttpServerRestarting,
    PlayerRestricted(usize),
    ServersRequested(Box<[AdminServerDto]>),
    ReferrersRequested(Box<[(Referrer, f32)]>),
    RestartRequested,
    RedirectRequested(Option<ServerId>),
    RedirectSet(Option<ServerId>),
    SeriesRequested(Arc<[(UnixTime, MetricsDataPointDto)]>),
    SummaryRequested(MetricsSummaryDto),
    UserAgentsRequested(Box<[(UserAgentId, f32)]>),
    ProfilerSet(bool),
}

/// Team related requests from the client to the server.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TeamRequest {
    Accept(PlayerId),
    Promote(PlayerId),
    Create(TeamName),
    Kick(PlayerId),
    Leave,
    Reject(PlayerId),
    Join(TeamId),
}

/// Team related update from server to client.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TeamUpdate {
    Promoted(PlayerId),
    Joining(TeamId),
    /// A complete enumeration of joiners, for the team captain only.
    Joiners(Box<[PlayerId]>),
    /// The following is for the joiner only, to indicate which teams they are joining.
    Joins(Box<[TeamId]>),
    /// A complete enumeration of team members, in order (first is captain).
    Members(Arc<[PlayerId]>),
    Accepted(PlayerId),
    Kicked(PlayerId),
    Rejected(PlayerId),
    Created(TeamId, TeamName),
    Left,
    AddedOrUpdated(Arc<[TeamDto]>),
    Removed(Arc<[TeamId]>),
}

/// Chat related request from client to server.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ChatRequest {
    Mute(PlayerId),
    Unmute(PlayerId),
    Send { message: String, whisper: bool },
}

/// Chat related update from server to client.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ChatUpdate {
    Sent,
    Received(Box<[Arc<MessageDto>]>),
    Muted(PlayerId),
    Unmuted(PlayerId),
}

/// Player related request from client to server.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlayerRequest {
    Report(PlayerId),
}

/// Player related update from server to client.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlayerUpdate {
    Reported(PlayerId),
    Updated {
        added: Arc<[PlayerDto]>,
        removed: Arc<[PlayerId]>,
        real_players: u32,
    },
}

/// Leaderboard related update from server to client.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LeaderboardUpdate {
    // The leaderboard contains high score players, but not teams, for prior periods.
    Updated(PeriodId, Arc<[LeaderboardDto]>),
}

/// Liveboard related update from server to client.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LiveboardUpdate {
    // The liveboard contains high score players and their teams in the current game.
    Updated {
        added: Arc<[LiveboardDto]>,
        removed: Arc<[PlayerId]>,
    },
}

/// Invitation related request from client to server.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InvitationRequest {
    CreateInvitation,
}

/// Invitation related update from server to client.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InvitationUpdate {
    InvitationCreated(InvitationId),
}

/// General request from client to server.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientRequest {
    SetAlias(PlayerAlias),
    TallyFps(f32),
    Trace { message: String },
}

/// General update from server to client.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientUpdate {
    SessionCreated {
        arena_id: ArenaId,
        server_id: Option<ServerId>,
        session_id: SessionId,
        player_id: PlayerId,
    },
    AliasSet(PlayerAlias),
    FpsTallied,
    Traced,
}

/// General update from server to client.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SystemUpdate {
    Added(Arc<[ServerDto]>),
    Removed(Arc<[ServerId]>),
}
