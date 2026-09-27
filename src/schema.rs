//! `open-LiveKit`のシグナリングGraphQLスキーマ(型定義のみ)。
//!
//! `open-tv-chat/PORTING.md`「6. RPoem GraphQL Subscriptionsスキーマ設計」で
//! 決めた初期案をSDL文字列として持つ。実装フェーズ1のスコープは型定義の
//! 登録(RPoemのFederation合成に載せられることの確認)までで、リゾルバは
//! 別モジュール([`crate::resolvers`])に最小ダミーとして置く。

/// このサブグラフ(`open-livekit`)が公開するSDL。
pub const SDL: &str = r#"
type Room {
  id: ID!
  name: String!
  participants: [Participant!]!
  createdAt: DateTime!
}

type Participant {
  id: ID!
  displayName: String!
  tracks: [Track!]!
  connectionMode: ConnectionMode!
}

enum ConnectionMode {
  RELAY
  P2P
}

type Track {
  id: ID!
  kind: TrackKind!
  ownerParticipantId: ID!
  sourceLanguage: String
  targetLanguage: String
}

enum TrackKind {
  AUDIO
  VIDEO
  TRANSLATED_AUDIO
  CAPTION
}

type CaptionEvent {
  participantId: ID!
  targetLanguage: String!
  text: String!
  timestamp: DateTime!
}

type Mutation {
  joinRoom(roomName: String!, displayName: String!): Participant!
  leaveRoom(participantId: ID!): Boolean!
  publishTrack(participantId: ID!, kind: TrackKind!): Track!
  unpublishTrack(trackId: ID!): Boolean!
  requestP2PMode(roomId: ID!, participantId: ID!): Boolean!
  respondP2PModeRequest(roomId: ID!, accept: Boolean!): Boolean!
  requestTranslation(participantId: ID!, targetLanguages: [String!]!): Boolean!
}

type Subscription {
  roomStateChanged(roomId: ID!): Room!
  trackPublished(roomId: ID!): Track!
  trackUnpublished(roomId: ID!): ID!
  captionReceived(roomId: ID!, targetLanguage: String!): CaptionEvent!
}
"#;

#[cfg(test)]
mod tests {
    use super::SDL;
    use open_runo_federation::sdl::parse_service_sdl;

    /// RPoemのFederation合成エンジンがこのSDLを1つのサブグラフとして
    /// 正しく型/フィールド抽出できることを確認する(型のみ・実装フェーズ1の
    /// 完了条件)。
    #[test]
    fn sdl_parses_as_service_schema() {
        let schema = parse_service_sdl("open-livekit", SDL).expect("SDL must parse");
        assert_eq!(schema.service_name, "open-livekit");
        for expected_type in [
            "Room",
            "Participant",
            "Track",
            "CaptionEvent",
            "Mutation",
            "Subscription",
        ] {
            assert!(
                schema.types.contains_key(expected_type),
                "expected type `{expected_type}` to be present in parsed schema"
            );
        }

        let mutation_fields = &schema.types["Mutation"];
        assert!(mutation_fields.contains("joinRoom"));
        assert!(mutation_fields.contains("requestTranslation"));

        let subscription_fields = &schema.types["Subscription"];
        assert!(subscription_fields.contains("captionReceived"));
    }
}
