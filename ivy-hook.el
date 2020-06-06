;; Here's how to open the connection
(setq ivy-ktrl-conn
      (open-network-stream
       "ivy-ktrl-conn"
       "*ivy-ktrl-conn*"
       "127.0.0.1"
       7331))

;; Here's how to send a message
;; (process-send-string ivy-ktrl-conn "hello world\n")

;; Here's how to kill it
;; (process-send-eof ivy-ktrl-conn)

(defun ivy-pre-read (orig-fun &rest args)
  (process-send-string ivy-ktrl-conn "in\n"))

(advice-add 'ivy-read :before #'ivy-pre-read)

(defun ivy-post-read ()
  (process-send-string ivy-ktrl-conn "out\n"))

(advice-add 'ivy--cleanup :after #'ivy-post-read)

;; remove the hooks
(advice-remove 'ivy-read #'ivy-pre-read)
(advice-remove 'ivy--cleanup #'ivy-post-read)
