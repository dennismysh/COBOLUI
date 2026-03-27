      *> ir-paragraph.cpy - Paragraph table
      *> Maps Rust: Paragraph { name, statements: Vec<Statement> }
      *> Statements stored in ir-statement.cpy; paragraph refs
      *> index range into the global statement table.

       01  WS-PARAGRAPH-TABLE.
           05  WS-PARA-COUNT         PIC 9(4) VALUE 0.
           05  WS-PARA-ENTRY OCCURS 100 TIMES.
               10  PARA-NAME         PIC X(30).
               10  PARA-STMT-START   PIC 9(6) VALUE 0.
               10  PARA-STMT-COUNT   PIC 9(4) VALUE 0.
