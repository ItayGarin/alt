;; Here's how to open the connection
(setq alt-conn
      (open-network-stream
       "alt-conn"
       "alt-conn"
       "127.0.0.1"
       7331))

;; Here's how to send a message
;; (process-send-string alt-conn "hello world\n")

;; Here's how to kill it
;; (process-send-eof alt-conn)

(defun alt--is-open-p ()
  (eq 'open (process-status alt-conn)))

(defun alt--ivy-on-hook (orig-fun &rest args)
  (when (alt--is-open)
    (process-send-string alt-conn "in\n")))

(defun alt--ivy-off-hook ()
  (when (alt--is-open)
    (process-send-string alt-conn "out\n")))

;; add the hooks
(advice-add 'ivy-read :before #'alt--ivy-on-hook)
(advice-add 'ivy--cleanup :after #'alt--ivy-off-hook)

;; remove the hooks
(advice-remove 'ivy-read #'alt--ivy-on-hook)
(advice-remove 'ivy--cleanup #'alt--ivy-off-hook)

;; -------------------------------------
;;
;; Here's how to hook evil modes -
;;
;; (defun my/enter-insert-state-hook()
;;   (interactive)
;;   (shell-command (concat "issw " issw_default_lang_source)))
;;
;; (defun my/exit-insert-state-hook()
;;   (interactive)
;;   (setq issw_default_lang_source (shell-command-to-string "issw"))
;;   (shell-command "issw com.apple.keylayout.US"))
;;
;; (add-hook 'evil-insert-state-entry-hook 'my/enter-insert-state-hook)
;; (add-hook 'evil-insert-state-exit-hook  'my/exit-insert-state-hook)
;; (add-hook 'evil-replace-state-entry-hook 'my/enter-insert-state-hook)
;; (add-hook 'evil-replace-state-exit-hook  'my/exit-insert-state-hook)
