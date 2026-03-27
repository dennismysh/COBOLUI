      *> ir-handler.cpy - Handler mapping table
      *> Maps Rust: Handler { name, paragraph_name }

       01  WS-HANDLER-TABLE.
           05  WS-HANDLER-COUNT      PIC 9(4) VALUE 0.
           05  WS-HANDLER-ENTRY OCCURS 100 TIMES.
               10  HANDLER-NAME      PIC X(30).
               10  HANDLER-PARA-NAME PIC X(30).
