;;;; A overview of the AST in a Lisp-like syntax.
;;;; ==================================================
;;;;
;;;; This is a visualisation of the parse tree of the code in the README.

(shebang "/usr/bin/env sylan")

(package public main

  (import (. io (println print)))

  (interface ToString () ()
    (method public String toString () ()))

  (interface Concatenate (T (= Result Self))
    (method public Result concatenate () ((T y))))

  (class Account () (ToString (Concatenate Account))
    (var public String firstName)
    (var public String lastName)
    (var public String ageInYears)

    (constructor Account ((String firstName) (String lastName))
      (println "instantiating an Account...")
      (super (= firstName firstName)
             (= lastName lastName)
             (= ageInYears 35)))

    (method override public String toString () ()
      `{(. this firstName)} {(. this lastName)} is {(. this ageInYears)} years old`)

    (method override public Account concatenate () ((Account a))
      (var String firstName ((. firstName concat) (. a firstName)))
      (var String lastName ((. lastName concat) (. a lastName)))

      (Account (= firstName firstName)
               (= lastName lastName)
               (= ageInYears (+ (. this ageInYears) (. a ageInYears)))))

    (get override public String name
      `{(. this firstName)} {(. this lastName)}`))

  (extends Account ((Concatenate Self (= Result String)))
    (method override public String concatenate () ((Account a))
      `{(. this firstName)} {(. a firstName)}`))

  (type Person Account)
  (type Showable ToString)

  (var int maxBound 5)

  (var (Function int () (int)) factorial (-> ((int n))
    (switch n
      (case (0, 1)
        1)
      (default
        (if (< n 0)
          (throw (Exception "n cannot be less than 0")))
        (factorial (* n (- n 1)))))))

  (package counter
    (class public Increment () ())
    (class public Reset () ())
    (class public Get () ())

    (var public (Function Task () (int)) (-> ((int n 0))
      (Task (-> ()
        (for ()
          (select
            (case Increment
              (Counter (+ n 1)))
            (case Reset
              (Counter 0))
            (case Get
              ((. sender send) n))
            (timeout (seconds 10)
              (throw (Exception "timed out!"))))))))))

  (var (Function void () ()) closureDemo (-> ()
    (var int x 5)

    (var Account account1 (Account (= firstName "Tom")
                                   (= lastName "Smith")
                                   (= ageInYears 15)))

    (var String firstName "Tom")
    (var String lastName "Smith")
    (var int age 25)
    (var Account account2 (Account (= firstName firstName)
                                   (= lastName lastName)
                                   (= ageInYears age)))

    (var (Function void () (Account)) f (-> ((Account a))
      (println ((. a toString)))))

    (f account1)
    (f (account2 (= firstName "Emma")))

    (var (Function Account Account) g (-> ((Account a))
      (println "returning an account")
      a))

    (var Account z (g account1))))

  (var (Function void () ()) demoLiterals (-> ()
    (var int a 5)
    (var uint b 5)
    (var decimal c 10.0)

    (var byte d 5u8)
    (var uint16 e 11u16)
    (var uint32 f 12u32)
    (var uint64 g 13u64)
    (var int8 h 15s8)
    (var short i 13s16)
    (var int32 j 7s32)
    (var long k 7s64)
    (var float l 12f16)
    (var double m 8f32)))

  (var (Function N ((N Add)) (N)) double (-> ((N n))
    (+ n n)))

  (var (Function void () ()) demoIteration (-> ()
    ((. (List 1 2 3) forEach) (-> (n)
      (println `{n}`)))

    ((. (List 1 2 3) map) double)

    (var int fact (for ((n 20) (result 0))
      (if (<= n 0)
        result
        (continue (- n 1) (* n result)))))

    (println `factorial: {fact}`)))

  (var (Function (Optional int) () ()) demoContexts (-> ()
    (do
      (<- int a (some 5))
      (doSomething)
      (<- int a (empty))
      (willNotBeRun))))

  ;;; Top-level code is allowed, but only in the main package. Code in other packages must be in
  ;;; functions or methods.

  (var Task c ((. counter create)))
  (times 5 (-> ()
    ((. c send) ((. counter Increment)))))

  ((. c send) ((. counter Get)))
  ((. c send) ((. counter Increment)))
  ((. c send) ((. counter Get)))

  (times 2 (-> ()
    (select
      (case (as Int n)
        (println `{n}`)))))

  (print """
    Multline
    strings
  """)

  (var int x (begin
    (println "Returning 5 to be bound as x...")
    5)))

  (print `{x}`))
