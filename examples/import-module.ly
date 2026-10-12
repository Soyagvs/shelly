# Import loads only exports and does not run module side effects. #
import "module.ly"

greet "Ada"
shellout "Imported constant: $$GREETING"
