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
    (= public String firstName)
    (= public String lastName)
    (= public String ageInYears)

    (constructor Account ((String firstName) (String lastName))
      (println "instantiating an Account...")
      (super (= firstName firstName)
             (= lastName lastName)
             (= ageInYears 35)))

    (method override public String toString () ()
      `{(. this firstName)} {(. this lastName)} is {(. this ageInYears)} years old`)

    (method override public Account concatenate () ((Account a))
      (= String firstName ((. firstName concat) (. a firstName)))
      (= String lastName ((. lastName concat) (. a lastName)))

      (Account (= firstName firstName)
               (= lastName lastName)
               (= ageInYears (+ (. this ageInYears) (. a ageInYears)))))

    (get override public String name
      `{(. this firstName)} {(. this lastName)}`))

  (extends Account ((Concatenate Self (= Result String)))
    (method override public String concatenate () ((Account a))
      `{(. this firstName)} {(. a firstName)}`))

  (= Person Account)
  (= Showable ToString)

  (= int maxBound 5)

  (= (Function int () (int)) factorial (-> ((int n))
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

    (= public (Function Task () (int)) (-> ((int n 0))
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

  (= (Function void () ()) closureDemo (-> ()
    (= int x 5)

    (= Account account1 (Account (= firstName "Tom")
                                 (= lastName "Smith")
                                 (= ageInYears 15)))

    (= String firstName "Tom")
    (= String lastName "Smith")
    (= int age 25)
    (= Account account2 (Account (= firstName firstName)
                                 (= lastName lastName)
                                 (= ageInYears age)))

    (= (Function void () (Account)) f (-> ((Account a))
      (println ((. a toString)))))

    (f account1)
    (f (account2 (= firstName "Emma")))

    (= (Function Account Account) g (-> ((Account a))
      (println "returning an account")
      a))

    (= Account z (g account1))))

  (= (Function void () ()) demoLiterals (-> ()
    (= int a 5)
    (= uint b 5)
    (= decimal c 10.0)

    (= byte d 5u8)
    (= uint16 e 11u16)
    (= uint32 f 12u32)
    (= uint64 g 13u64)
    (= int8 h 15s8)
    (= short i 13s16)
    (= int32 j 7s32)
    (= long k 7s64)
    (= float l 12f16)
    (= double m 8f32)))

  (= (Function N ((N Add)) (N)) double (-> ((N n))
    (+ n n)))

  (= (Function void () ()) demoIteration (-> ()
    ((. (List 1 2 3) forEach) (-> (n)
      (println `{n}`)))

    ((. (List 1 2 3) map) double)

    (= int fact (for ((n 20) (result 0))
      (if (<= n 0)
        result
        (continue (- n 1) (* n result)))))

    (println `factorial: {fact}`)))

  (= (Function (Optional int) () ()) demoContexts (-> ()
    (do
      (<- int a (some 5))
      (doSomething)
      (<- int a (empty))
      (willNotBeRun))))

  ;;; Top-level code is allowed, but only in the main package. Code in other packages must be in
  ;;; functions or methods.

  (= Task c ((. counter create)))
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

  (= int x ((-> ()
    (println "Returning 5 to be bound as x...")
    5)))

  (print `{x}`))
