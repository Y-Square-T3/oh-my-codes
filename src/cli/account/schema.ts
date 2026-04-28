import * as Schema from "effect/Schema"

export const AccountID = Schema.String.pipe(Schema.brand("AccountID"))
export type AccountID = Schema.Schema.Type<typeof AccountID>

export const WorkspaceID = Schema.String.pipe(Schema.brand("WorkspaceID"))
export type WorkspaceID = Schema.Schema.Type<typeof WorkspaceID>

export const AccessToken = Schema.String.pipe(Schema.brand("AccessToken"))
export type AccessToken = Schema.Schema.Type<typeof AccessToken>

export const RefreshToken = Schema.String.pipe(Schema.brand("RefreshToken"))
export type RefreshToken = Schema.Schema.Type<typeof RefreshToken>

export const DeviceCode = Schema.String.pipe(Schema.brand("DeviceCode"))
export type DeviceCode = Schema.Schema.Type<typeof DeviceCode>

export const UserCode = Schema.String.pipe(Schema.brand("UserCode"))
export type UserCode = Schema.Schema.Type<typeof UserCode>

export class AccountInfo extends Schema.Class<AccountInfo>("AccountInfo")({
  id: AccountID,
  email: Schema.String,
  url: Schema.String,
  active_workspace_id: Schema.NullOr(WorkspaceID),
}) {}

export class Workspace extends Schema.Class<Workspace>("Workspace")({
  id: WorkspaceID,
  name: Schema.String,
  isAdmin: Schema.Boolean,
}) {}

export class Login extends Schema.Class<Login>("Login")({
  code: DeviceCode,
  user: UserCode,
  url: Schema.String,
  server: Schema.String,
  expiry: Schema.Number,
  interval: Schema.Number,
}) {}

export class PollSuccess extends Schema.TaggedClass<PollSuccess>()("PollSuccess", {
  email: Schema.String,
}) {}

export class PollPending extends Schema.TaggedClass<PollPending>()("PollPending", {}) {}

export class PollSlow extends Schema.TaggedClass<PollSlow>()("PollSlow", {}) {}

export class PollExpired extends Schema.TaggedClass<PollExpired>()("PollExpired", {}) {}

export class PollDenied extends Schema.TaggedClass<PollDenied>()("PollDenied", {}) {}

export class PollError extends Schema.TaggedClass<PollError>()("PollError", {
  cause: Schema.Defect,
}) {}

export const PollResult = Schema.Union(PollSuccess, PollPending, PollSlow, PollExpired, PollDenied, PollError)
export type PollResult = Schema.Schema.Type<typeof PollResult>

export class AccountRepoError extends Schema.TaggedClass<AccountRepoError>()("AccountRepoError", {
  message: Schema.String,
  cause: Schema.optional(Schema.Defect),
}) {}

export class AccountServiceError extends Schema.TaggedClass<AccountServiceError>()("AccountServiceError", {
  message: Schema.String,
  cause: Schema.optional(Schema.Defect),
}) {}

export class AccountTransportError extends Schema.TaggedClass<AccountTransportError>()("AccountTransportError", {
  method: Schema.String,
  url: Schema.String,
  description: Schema.optional(Schema.String),
  cause: Schema.optional(Schema.Defect),
}) {}

export type AccountError = AccountRepoError | AccountServiceError | AccountTransportError

export class DeviceAuthResponse extends Schema.Class<DeviceAuthResponse>("DeviceAuthResponse")({
  device_code: Schema.String,
  user_code: Schema.String,
  verification_uri_complete: Schema.String,
  expires_in: Schema.Number,
  interval: Schema.Number,
}) {}

export class DeviceTokenSuccess extends Schema.Class<DeviceTokenSuccess>("DeviceTokenSuccess")({
  access_token: Schema.String,
  refresh_token: Schema.String,
  token_type: Schema.Literal("Bearer"),
  expires_in: Schema.Number,
}) {}

export class DeviceTokenError extends Schema.Class<DeviceTokenError>("DeviceTokenError")({
  error: Schema.String,
  error_description: Schema.String,
}) {
  toPollResult(): PollResult {
    if (this.error === "authorization_pending") return new PollPending()
    if (this.error === "slow_down") return new PollSlow()
    if (this.error === "expired_token") return new PollExpired()
    if (this.error === "access_denied") return new PollDenied()
    return new PollError({ cause: this.error })
  }
}

export const DeviceToken = Schema.Union(DeviceTokenSuccess, DeviceTokenError)

export class User extends Schema.Class<User>("User")({
  id: AccountID,
  email: Schema.String,
}) {}

export class TokenRefreshResponse extends Schema.Class<TokenRefreshResponse>("TokenRefreshResponse")({
  access_token: Schema.String,
  refresh_token: Schema.String,
  expires_in: Schema.Number,
}) {}
