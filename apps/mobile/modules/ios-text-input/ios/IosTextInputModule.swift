import ExpoModulesCore
import UIKit

public class IosTextInputModule: Module {
  public func definition() -> ModuleDefinition {
    Name("IosTextInput")

    View(IosTextInputView.self) {
      Events(
        "onTextChange",
        "onInputFocus",
        "onInputBlur",
        "onInputSubmitEditing",
        "onInputEndEditing",
        "onInputChange",
        "onInputSelectionChange",
        "onInputKeyPress",
        "onContentSizeChange",
        "onInputPress"
      )

      AsyncFunction("focus") { (view: IosTextInputView) in
        view.focus()
      }

      AsyncFunction("blur") { (view: IosTextInputView) in
        view.blur()
      }

      AsyncFunction("clear") { (view: IosTextInputView) in
        view.clear()
      }

      AsyncFunction("isFocused") { (view: IosTextInputView) -> Bool in
        return view.isFocused()
      }

      // MARK: - Core Props

      Prop("text") { (view: IosTextInputView, text: String) in
        view.setText(text)
      }

      Prop("multiline") { (view: IosTextInputView, multiline: Bool) in
        view.isMultiline = multiline
      }

      Prop("placeholder") { (view: IosTextInputView, placeholder: String?) in
        view.setPlaceholder(placeholder)
      }

      Prop("secureTextEntry") { (view: IosTextInputView, secure: Bool) in
        view.textField.isSecureTextEntry = secure
      }

      Prop("showDoneButton") { (view: IosTextInputView, show: Bool) in
        view.setShowDoneButton(show)
      }

      Prop("editable") { (view: IosTextInputView, editable: Bool) in
        view.setEditable(editable)
      }

      Prop("maxLength") { (view: IosTextInputView, length: Int?) in
        view.maxLength = length
      }

      // MARK: - Appearance

      Prop("textColor") { (view: IosTextInputView, color: UIColor) in
        view.setTextColor(color)
      }

      Prop("placeholderTextColor") { (view: IosTextInputView, color: UIColor?) in
        if let color = color {
          view.storedPlaceholderColor = color
          view.placeholderLabel.textColor = color
          if let text = view.storedPlaceholder {
            view.textField.attributedPlaceholder = NSAttributedString(
              string: text,
              attributes: [.foregroundColor: color]
            )
          }
        }
      }

      Prop("fontSize") { (view: IosTextInputView, size: Double) in
        view.setFontSize(CGFloat(size))
      }

      Prop("textAlign") { (view: IosTextInputView, value: String?) in
        switch value {
        case "center": view.setTextAlignment(.center)
        case "right":  view.setTextAlignment(.right)
        default:       view.setTextAlignment(.natural)
        }
      }

      Prop("selectionColor") { (view: IosTextInputView, color: UIColor?) in
        view.setSelectionColor(color)
      }

      Prop("caretHidden") { (view: IosTextInputView, hidden: Bool) in
        view.setCaretHidden(hidden)
      }

      // MARK: - Keyboard / Input Traits

      Prop("autoCapitalize") { (view: IosTextInputView, value: String?) in
        switch value {
        case "characters": view.setAutocapitalization(.allCharacters)
        case "words":      view.setAutocapitalization(.words)
        case "sentences":  view.setAutocapitalization(.sentences)
        case "none":       view.setAutocapitalization(.none)
        default:           view.setAutocapitalization(.sentences)
        }
      }

      Prop("autoCorrect") { (view: IosTextInputView, enabled: Bool) in
        view.setAutocorrection(enabled ? .yes : .no)
      }

      Prop("spellCheck") { (view: IosTextInputView, enabled: Bool) in
        view.setSpellChecking(enabled ? .yes : .no)
      }

      Prop("keyboardType") { (view: IosTextInputView, value: String?) in
        switch value {
        case "email-address":          view.setKeyboardType(.emailAddress)
        case "numeric":                view.setKeyboardType(.numberPad)
        case "decimal-pad":            view.setKeyboardType(.decimalPad)
        case "number-pad":             view.setKeyboardType(.numberPad)
        case "phone-pad":              view.setKeyboardType(.phonePad)
        case "url":                    view.setKeyboardType(.URL)
        case "ascii-capable":          view.setKeyboardType(.asciiCapable)
        case "numbers-and-punctuation": view.setKeyboardType(.numbersAndPunctuation)
        case "name-phone-pad":         view.setKeyboardType(.namePhonePad)
        case "twitter":                view.setKeyboardType(.twitter)
        case "web-search":             view.setKeyboardType(.webSearch)
        case "visible-password":       view.setKeyboardType(.asciiCapable)
        default:                       view.setKeyboardType(.default)
        }
      }

      Prop("returnKeyType") { (view: IosTextInputView, value: String?) in
        switch value {
        case "done":           view.setReturnKeyType(.done)
        case "go":             view.setReturnKeyType(.go)
        case "next":           view.setReturnKeyType(.next)
        case "search":         view.setReturnKeyType(.search)
        case "send":           view.setReturnKeyType(.send)
        case "emergency-call": view.setReturnKeyType(.emergencyCall)
        case "google":         view.setReturnKeyType(.google)
        case "join":           view.setReturnKeyType(.join)
        case "route":          view.setReturnKeyType(.route)
        case "yahoo":          view.setReturnKeyType(.yahoo)
        default:               view.setReturnKeyType(.default)
        }
      }

      Prop("keyboardAppearance") { (view: IosTextInputView, value: String?) in
        switch value {
        case "light": view.setKeyboardAppearance(.light)
        case "dark":  view.setKeyboardAppearance(.dark)
        default:      view.setKeyboardAppearance(.default)
        }
      }

      Prop("enablesReturnKeyAutomatically") { (view: IosTextInputView, enabled: Bool) in
        view.setEnablesReturnKeyAutomatically(enabled)
      }

      Prop("showSoftInputOnFocus") { (view: IosTextInputView, show: Bool) in
        view.setShowSoftInputOnFocus(show)
      }

      // MARK: - Text Content / Autofill

      Prop("textContentType") { (view: IosTextInputView, value: String?) in
        switch value {
        case "username":          view.setTextContentType(.username)
        case "password":          view.setTextContentType(.password)
        case "newPassword":       view.setTextContentType(.newPassword)
        case "emailAddress":      view.setTextContentType(.emailAddress)
        case "oneTimeCode":       view.setTextContentType(.oneTimeCode)
        case "name":              view.setTextContentType(.name)
        case "givenName":         view.setTextContentType(.givenName)
        case "familyName":        view.setTextContentType(.familyName)
        case "middleName":        view.setTextContentType(.middleName)
        case "namePrefix":        view.setTextContentType(.namePrefix)
        case "nameSuffix":        view.setTextContentType(.nameSuffix)
        case "nickname":          view.setTextContentType(.nickname)
        case "organizationName":  view.setTextContentType(.organizationName)
        case "jobTitle":          view.setTextContentType(.jobTitle)
        case "location":          view.setTextContentType(.location)
        case "fullStreetAddress": view.setTextContentType(.fullStreetAddress)
        case "streetAddressLine1": view.setTextContentType(.streetAddressLine1)
        case "streetAddressLine2": view.setTextContentType(.streetAddressLine2)
        case "addressCity":       view.setTextContentType(.addressCity)
        case "addressCityAndState": view.setTextContentType(.addressCityAndState)
        case "addressState":      view.setTextContentType(.addressState)
        case "postalCode":        view.setTextContentType(.postalCode)
        case "sublocality":       view.setTextContentType(.sublocality)
        case "countryName":       view.setTextContentType(.countryName)
        case "telephoneNumber":   view.setTextContentType(.telephoneNumber)
        case "creditCardNumber":  view.setTextContentType(.creditCardNumber)
        case "URL":               view.setTextContentType(.URL)
        case "none":              view.setTextContentType(.none)
        default:                  view.setTextContentType(nil)
        }
      }

      Prop("passwordRules") { (view: IosTextInputView, rules: String?) in
        view.setPasswordRules(rules)
      }

      // MARK: - Behavior

      Prop("submitBehavior") { (view: IosTextInputView, value: String?) in
        view.submitBehavior = value
      }

      Prop("autoFocus") { (view: IosTextInputView, focus: Bool) in
        if focus {
          DispatchQueue.main.async {
            view.focus()
          }
        }
      }

      Prop("contextMenuHidden") { (view: IosTextInputView, hidden: Bool) in
        view.setContextMenuHidden(hidden)
      }

      Prop("dismissOnTapOutside") { (view: IosTextInputView, dismiss: Bool) in
        view.setDismissOnTapOutside(dismiss)
      }

      Prop("clearTextOnFocus") { (view: IosTextInputView, value: Bool) in
        view.clearTextOnFocus = value
      }

      Prop("selectTextOnFocus") { (view: IosTextInputView, value: Bool) in
        view.selectTextOnFocus = value
      }

      Prop("selection") { (view: IosTextInputView, value: [String: Int]?) in
        if let start = value?["start"], let end = value?["end"] {
          DispatchQueue.main.async {
            view.setSelection(start: start, end: end)
          }
        }
      }

      // MARK: - Single-line Only (UITextField)

      Prop("clearButtonMode") { (view: IosTextInputView, value: String?) in
        switch value {
        case "always":         view.textField.clearButtonMode = .always
        case "while-editing":  view.textField.clearButtonMode = .whileEditing
        case "unless-editing": view.textField.clearButtonMode = .unlessEditing
        default:               view.textField.clearButtonMode = .never
        }
      }

      // MARK: - Multiline Only (UITextView)

      Prop("scrollEnabled") { (view: IosTextInputView, enabled: Bool) in
        view.isInputScrollEnabled = enabled
      }

      Prop("numberOfLines") { (view: IosTextInputView, lines: Int?) in
        view.setNumberOfLines(lines)
      }

      Prop("smartInsertDelete") { (view: IosTextInputView, enabled: Bool) in
        view.setSmartInsertDelete(enabled)
      }

      Prop("mostRecentEventCount") { (view: IosTextInputView, count: Int) in
        view.mostRecentEventCount = count
      }

      OnViewDidUpdateProps { view in
        view.applyPendingTextUpdate()
      }
    }
  }
}

