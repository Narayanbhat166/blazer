use crate::app::client::network::types::UserDetails as NetworkUserDetails;
use crate::app::client::types::UserDetails as AppUserDetails;
use crate::grpc::server::UserDetails as GrpcUserDetails;

impl From<NetworkUserDetails> for GrpcUserDetails {
    fn from(network_user: NetworkUserDetails) -> Self {
        Self {
            user_id: network_user.user_id,
            user_name: network_user.user_name,
            games_played: network_user.games_played,
            rank: network_user.rank,
        }
    }
}

impl From<GrpcUserDetails> for NetworkUserDetails {
    fn from(grpc_user: GrpcUserDetails) -> Self {
        Self {
            user_id: grpc_user.user_id,
            user_name: grpc_user.user_name,
            games_played: grpc_user.games_played,
            rank: grpc_user.rank,
        }
    }
}

impl From<NetworkUserDetails> for AppUserDetails {
    fn from(network_user: NetworkUserDetails) -> Self {
        Self {
            user_id: network_user.user_id,
            user_name: network_user.user_name,
            games_played: network_user.games_played,
            rank: network_user.rank,
        }
    }
}
