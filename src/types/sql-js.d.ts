declare module "sql.js" {
  interface SqlJsStatic {
    Database: typeof Database
  }

  class Database {
    constructor(data?: ArrayLike<number> | Buffer | null)
    run(sql: string, params?: unknown): void
    exec(sql: string): Statement[]
    export(): Uint8Array
    close(): void
    getRowsModified(): number
    save(): Uint8Array
  }

  interface Statement {
    columns: string[]
    values: unknown[][]
  }

  interface InitSqlJsOptions {
    locateFile?: (filename: string) => string
  }

  export default function initSqlJs(options?: InitSqlJsOptions): Promise<SqlJsStatic>
  export type { SqlJsStatic, Database }
}
