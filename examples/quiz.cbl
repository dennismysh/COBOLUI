       IDENTIFICATION DIVISION.
       PROGRAM-ID. QUIZ.

       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01  APP-STATE.
           05  SCORE           PIC 9(2) VALUE 0.
           05  TOTAL-QS        PIC 9(2) VALUE 3.
           05  ANSWER-1        PIC X(20) VALUE "type here".
           05  ANSWER-2        PIC X(20) VALUE "type here".
           05  ANSWER-3        PIC X(20) VALUE "type here".
           05  RESULT-1        PIC X(30) VALUE "unanswered".
           05  RESULT-2        PIC X(30) VALUE "unanswered".
           05  RESULT-3        PIC X(30) VALUE "unanswered".
           05  FINAL-MSG       PIC X(60) VALUE "not graded".
           05  STATUS-MSG      PIC X(60) VALUE "Answer the questions".

       SCREEN SECTION.
       01  Q1-SCREEN.
           05  HEADER.
               10  TITLE       PIC X(40) VALUE "Question 1 of 3".
           05  QUESTION.
               10  Q-TEXT      PIC X(60) VALUE "What language does COBALT parse?".
           05  ANSWER-AREA.
               10  ANS-INPUT   PIC X(20) USING ANSWER-1.
           05  CONTROLS.
               10  CHECK-BTN   VALUE "Check" ON-ACTION PERFORM CHECK-Q1.
               10  NEXT-BTN    VALUE "Next" GO-TO-SCREEN Q2-SCREEN.
           05  FEEDBACK.
               10  FB-TEXT     PIC X(30) USING RESULT-1.
           05  STATUS-BAR.
               10  MSG-TEXT    PIC X(60) USING STATUS-MSG.

       01  Q2-SCREEN.
           05  HEADER.
               10  TITLE       PIC X(40) VALUE "Question 2 of 3".
           05  QUESTION.
               10  Q-TEXT      PIC X(60) VALUE "What does PIC 9 define?".
           05  ANSWER-AREA.
               10  ANS-INPUT   PIC X(20) USING ANSWER-2.
           05  CONTROLS.
               10  CHECK-BTN   VALUE "Check" ON-ACTION PERFORM CHECK-Q2.
               10  NEXT-BTN    VALUE "Next" GO-TO-SCREEN Q3-SCREEN.
               10  PREV-BTN    VALUE "Previous" GO-TO-SCREEN Q1-SCREEN.
           05  FEEDBACK.
               10  FB-TEXT     PIC X(30) USING RESULT-2.
           05  STATUS-BAR.
               10  MSG-TEXT    PIC X(60) USING STATUS-MSG.

       01  Q3-SCREEN.
           05  HEADER.
               10  TITLE       PIC X(40) VALUE "Question 3 of 3".
           05  QUESTION.
               10  Q-TEXT      PIC X(60) VALUE "If A=3 B=4, what is A+B*2?".
           05  ANSWER-AREA.
               10  ANS-INPUT   PIC X(20) USING ANSWER-3.
           05  CONTROLS.
               10  CHECK-BTN   VALUE "Check" ON-ACTION PERFORM CHECK-Q3.
               10  FINISH-BTN  VALUE "Results" GO-TO-SCREEN RESULTS-SCREEN.
               10  PREV-BTN    VALUE "Previous" GO-TO-SCREEN Q2-SCREEN.
           05  FEEDBACK.
               10  FB-TEXT     PIC X(30) USING RESULT-3.
           05  STATUS-BAR.
               10  MSG-TEXT    PIC X(60) USING STATUS-MSG.

       01  RESULTS-SCREEN.
           05  HEADER.
               10  TITLE       PIC X(40) VALUE "Quiz Results".
           05  SCORE-AREA.
               10  SCORE-DISP  PIC 9(2) USING SCORE.
               10  TOTAL-DISP  PIC 9(2) USING TOTAL-QS.
               10  FINAL-DISP  PIC X(60) USING FINAL-MSG.
           05  ANSWERS.
               10  R1-DISP     PIC X(30) USING RESULT-1.
               10  R2-DISP     PIC X(30) USING RESULT-2.
               10  R3-DISP     PIC X(30) USING RESULT-3.
           05  CONTROLS.
               10  RETRY-BTN   VALUE "Retry" GO-TO-SCREEN Q1-SCREEN.
               10  GRADE-BTN   VALUE "Grade" ON-ACTION PERFORM CALC-GRADE.
           05  STATUS-BAR.
               10  MSG-TEXT    PIC X(60) USING STATUS-MSG.

       PROCEDURE DIVISION.
       MAIN-LOOP.
           STOP RUN.

       CHECK-Q1.
           EVALUATE ANSWER-1
               WHEN "COBOL"
                   MOVE "Correct!" TO RESULT-1
                   ADD 1 TO SCORE
               WHEN OTHER
                   MOVE "Wrong - answer is COBOL" TO RESULT-1
           END-EVALUATE.
           MOVE "Answer checked" TO STATUS-MSG.

       CHECK-Q2.
           EVALUATE ANSWER-2
               WHEN "NUMERIC"
                   MOVE "Correct!" TO RESULT-2
                   ADD 1 TO SCORE
               WHEN "NUMBER"
                   MOVE "Correct!" TO RESULT-2
                   ADD 1 TO SCORE
               WHEN OTHER
                   MOVE "Wrong - answer is NUMERIC" TO RESULT-2
           END-EVALUATE.
           MOVE "Answer checked" TO STATUS-MSG.

       CHECK-Q3.
           EVALUATE ANSWER-3
               WHEN "11"
                   MOVE "Correct!" TO RESULT-3
                   ADD 1 TO SCORE
               WHEN OTHER
                   MOVE "Wrong - answer is 11" TO RESULT-3
           END-EVALUATE.
           MOVE "Answer checked" TO STATUS-MSG.

       CALC-GRADE.
           EVALUATE SCORE
               WHEN 3
                   MOVE "Perfect! A+" TO FINAL-MSG
               WHEN 2
                   MOVE "Good! B" TO FINAL-MSG
               WHEN 1
                   MOVE "Needs work. C" TO FINAL-MSG
               WHEN 0
                   MOVE "Study more! F" TO FINAL-MSG
               WHEN OTHER
                   MOVE "Scored" TO FINAL-MSG
           END-EVALUATE.
           MOVE "Grade calculated" TO STATUS-MSG.
