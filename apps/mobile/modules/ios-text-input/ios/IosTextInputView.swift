import ExpoModulesCore
import UIKit

class InputTextField: UITextField {
  var isContextMenuHidden = false
  var isSoftInputVisible = true

  override var inputView: UIView? {
    get { isSoftInputVisible ? super.inputView : UIView() }
    set { super.inputView = newValue }
  }

  override func canPerformAction(_ action: Selector, withSender sender: Any?) -> Bool {
    isContextMenuHidden ? false : super.canPerformAction(action, withSender: sender)
  }
}

class InputTextView: UITextView {
  var isContextMenuHidden = false
  var isSoftInputVisible = true

  override var inputView: UIView? {
    get { isSoftInputVisible ? super.inputView : UIView() }
    set { super.inputView = newValue }
  }

  override func canPerformAction(_ action: Selector, withSender sender: Any?) -> Bool {
    isContextMenuHidden ? false : super.canPerformAction(action, withSender: sender)
  }
}

// MARK: - Main View

class IosTextInputView: ExpoView, UITextFieldDelegate, UITextViewDelegate {
  let textField = InputTextField()
  let textView = InputTextView()
  let placeholderLabel = UILabel()

  // MARK: Event Dispatchers

  let onTextChange = EventDispatcher()
  let onInputFocus = EventDispatcher()
  let onInputBlur = EventDispatcher()
  let onInputSubmitEditing = EventDispatcher()
  let onInputEndEditing = EventDispatcher()
  let onInputChange = EventDispatcher()
  let onInputSelectionChange = EventDispatcher()
  let onInputKeyPress = EventDispatcher()
  let onContentSizeChange = EventDispatcher()
  let onInputPress = EventDispatcher()

  // MARK: Stored Properties

  private var doneToolbar: UIToolbar?
  private var windowTapGesture: UITapGestureRecognizer?
  private var inputTapGesture: UITapGestureRecognizer?
  var dismissOnTapOutside: Bool = true

  var maxLength: Int?
  var storedPlaceholderColor: UIColor?
  var storedPlaceholder: String?
  var lastContentSize: CGSize = .zero

  var submitBehavior: String?
  var clearTextOnFocus: Bool = false
  var selectTextOnFocus: Bool = false

  var nativeEventCount = 0
  var mostRecentEventCount = 0
  var pendingText: String?

  var isMultiline: Bool = false {
    didSet {
      if isMultiline != oldValue {
        updateInputMode()
      }
    }
  }

  var isInputScrollEnabled: Bool = false {
    didSet {
      if isInputScrollEnabled != oldValue {
        textView.isScrollEnabled = isInputScrollEnabled
        if isMultiline && !isInputScrollEnabled {
          notifyContentSizeChange()
        }
      }
    }
  }

  // MARK: Computed

  private var effectiveSubmitBehavior: String {
    if let behavior = submitBehavior { return behavior }
    return isMultiline ? "newline" : "blurAndSubmit"
  }

  // MARK: - Initialization

  required init(appContext: AppContext? = nil) {
    super.init(appContext: appContext)
    clipsToBounds = true

    // Input tap gesture for onPress
    let tap = UITapGestureRecognizer(target: self, action: #selector(handleInputTap))
    tap.cancelsTouchesInView = false
    addGestureRecognizer(tap)
    inputTapGesture = tap

    // UITextField setup
    textField.delegate = self
    textField.addTarget(self, action: #selector(textFieldDidChange), for: .editingChanged)
    addSubview(textField)

    // UITextView setup
    textView.delegate = self
    textView.font = UIFont.systemFont(ofSize: 17)
    textView.backgroundColor = .clear
    textView.textContainerInset = .zero
    textView.textContainer.lineFragmentPadding = 0
    textView.isScrollEnabled = false
    textView.isHidden = true
    addSubview(textView)

    // Placeholder label for UITextView (UITextField has built-in placeholder)
    placeholderLabel.font = textView.font
    placeholderLabel.textColor = UIColor.placeholderText
    placeholderLabel.numberOfLines = 0
    placeholderLabel.isHidden = true
    addSubview(placeholderLabel)
  }

  // MARK: - Layout

  override func layoutSubviews() {
    super.layoutSubviews()

    if textField.frame != bounds { textField.frame = bounds }
    if textView.frame != bounds { textView.frame = bounds }

    // Position placeholder to match UITextView's text origin (top-left aligned)
    let inset = textView.textContainerInset
    let padding = textView.textContainer.lineFragmentPadding
    let placeholderX = inset.left + padding
    let placeholderY = inset.top
    let placeholderWidth = bounds.width - placeholderX - inset.right - padding
    let placeholderHeight = placeholderLabel.sizeThatFits(
      CGSize(width: placeholderWidth, height: CGFloat.greatestFiniteMagnitude)
    ).height
    let newFrame = CGRect(x: placeholderX, y: placeholderY, width: placeholderWidth, height: placeholderHeight)
    if placeholderLabel.frame != newFrame {
      placeholderLabel.frame = newFrame
    }
  }

  override var intrinsicContentSize: CGSize {
    if isMultiline && !isInputScrollEnabled {
      let fittingSize = textView.sizeThatFits(
        CGSize(width: bounds.width > 0 ? bounds.width : UIView.layoutFittingExpandedSize.width,
          height: UIView.layoutFittingExpandedSize.height)
      )
      return fittingSize
    }
    return super.intrinsicContentSize
  }

  // MARK: - Input Mode Toggle

  private func updateInputMode() {
    textField.isHidden = isMultiline
    textView.isHidden = !isMultiline
    updatePlaceholderVisibility()

    if isMultiline {
      textView.text = textField.text
    } else {
      textField.text = textView.text
    }
  }

  // MARK: - Shared Setters

  func setText(_ newText: String) {
    pendingText = newText
  }

  func applyPendingTextUpdate() {
    guard let newText = pendingText else { return }
    pendingText = nil // clear it out
    if mostRecentEventCount < nativeEventCount {
      return
    }

    if isMultiline {
      if textView.markedTextRange != nil { return }
      if textView.text != newText {
        textView.text = newText
        updatePlaceholderVisibility()
        notifyContentSizeChange()
      }
    } else {
      if textField.markedTextRange != nil { return }
      if textField.text != newText {
        textField.text = newText
      }
    }
  }

  func setPlaceholder(_ placeholder: String?) {
    storedPlaceholder = placeholder
    textField.placeholder = placeholder
    placeholderLabel.text = placeholder
    if let color = storedPlaceholderColor, let text = placeholder {
      textField.attributedPlaceholder = NSAttributedString(
        string: text, attributes: [.foregroundColor: color]
      )
    }
  }

  func setTextColor(_ color: UIColor) {
    textField.textColor = color
    textView.textColor = color
  }

  func setFontSize(_ size: CGFloat) {
    let font = UIFont.systemFont(ofSize: size)
    textField.font = font
    textView.font = font
    placeholderLabel.font = font
    if isMultiline { notifyContentSizeChange() }
  }

  func setTextAlignment(_ alignment: NSTextAlignment) {
    textField.textAlignment = alignment
    textView.textAlignment = alignment
  }

  func setEditable(_ editable: Bool) {
    textField.isEnabled = editable
    textView.isEditable = editable
  }

  func setAutocapitalization(_ type: UITextAutocapitalizationType) {
    textField.autocapitalizationType = type
    textView.autocapitalizationType = type
  }

  func setAutocorrection(_ type: UITextAutocorrectionType) {
    textField.autocorrectionType = type
    textView.autocorrectionType = type
  }

  func setSpellChecking(_ type: UITextSpellCheckingType) {
    textField.spellCheckingType = type
    textView.spellCheckingType = type
  }

  func setKeyboardType(_ type: UIKeyboardType) {
    textField.keyboardType = type
    textView.keyboardType = type
  }

  func setReturnKeyType(_ type: UIReturnKeyType) {
    textField.returnKeyType = type
    textView.returnKeyType = type
  }

  func setTextContentType(_ type: UITextContentType?) {
    textField.textContentType = type
    textView.textContentType = type
  }

  func setKeyboardAppearance(_ appearance: UIKeyboardAppearance) {
    textField.keyboardAppearance = appearance
    textView.keyboardAppearance = appearance
  }

  func setEnablesReturnKeyAutomatically(_ enabled: Bool) {
    textField.enablesReturnKeyAutomatically = enabled
    textView.enablesReturnKeyAutomatically = enabled
  }

  func setCaretHidden(_ hidden: Bool) {
    let color: UIColor = hidden ? .clear : tintColor
    textField.tintColor = color
    textView.tintColor = color
  }

  func setSelectionColor(_ color: UIColor?) {
    textField.tintColor = color
    textView.tintColor = color
  }

  func setContextMenuHidden(_ hidden: Bool) {
    textField.isContextMenuHidden = hidden
    textView.isContextMenuHidden = hidden
  }

  func setDismissOnTapOutside(_ dismiss: Bool) {
    dismissOnTapOutside = dismiss
  }

  func setShowSoftInputOnFocus(_ show: Bool) {
    textField.isSoftInputVisible = show
    textView.isSoftInputVisible = show
    // Reload input views if already focused
    if textField.isFirstResponder {
      textField.reloadInputViews()
    }
    if textView.isFirstResponder {
      textView.reloadInputViews()
    }
  }

  func setSelection(start: Int, end: Int) {
    if isMultiline {
      if textView.markedTextRange != nil { return }
      let length = (textView.text as NSString?)?.length ?? 0
      let clampedStart = min(max(start, 0), length)
      let clampedEnd = min(max(end, 0), length)
      textView.selectedRange = NSRange(location: clampedStart, length: clampedEnd - clampedStart)
    } else {
      if textField.markedTextRange != nil { return }
      guard let startPos = textField.position(from: textField.beginningOfDocument, offset: start),
            let endPos = textField.position(from: textField.beginningOfDocument, offset: end) else { return }
      textField.selectedTextRange = textField.textRange(from: startPos, to: endPos)
    }
  }

  func setNumberOfLines(_ lines: Int?) {
    textView.textContainer.maximumNumberOfLines = lines ?? 0
  }

  func setSmartInsertDelete(_ enabled: Bool) {
    textView.smartInsertDeleteType = enabled ? .yes : .no
  }

  func setPasswordRules(_ rules: String?) {
    if let rules = rules {
      textField.passwordRules = UITextInputPasswordRules(descriptor: rules)
    } else {
      textField.passwordRules = nil
    }
  }

  // MARK: - Placeholder Visibility

  private func updatePlaceholderVisibility() {
    placeholderLabel.isHidden = !isMultiline || !(textView.text?.isEmpty ?? true)
  }

  // MARK: - Content Size Notification

  private func notifyContentSizeChange() {
    let size = textView.sizeThatFits(
      CGSize(width: bounds.width > 0 ? bounds.width : UIView.layoutFittingExpandedSize.width,
             height: UIView.layoutFittingExpandedSize.height)
    )

    if size != lastContentSize {
      lastContentSize = size
      invalidateIntrinsicContentSize()
      onContentSizeChange([
        "contentSize": ["width": size.width, "height": size.height]
      ])
    }
  }

  // MARK: - Key Press Helper

  private func fireKeyPress(_ string: String) {
    let key: String
    if string == "\n" {
      key = "Enter"
    } else if string.isEmpty {
      key = "Backspace"
    } else {
      key = string
    }
    onInputKeyPress(["key": key])
  }

  // MARK: - Tap Events

  @objc private func handleDoneButtonTap() {
    // Fire the submit event so React knows the user finished typing
    let text = isMultiline ? textView.text : textField.text
    onInputSubmitEditing(["text": text ?? ""])

    // Dismiss the keyboard
    blur()
  }

  @objc private func handleInputTap() {
    onInputPress([:])
  }

  // MARK: - Window Tap to Dismiss

  private func installWindowTapGesture() {
    guard dismissOnTapOutside else { return }
    guard windowTapGesture == nil, let window = self.window else { return }
    let gesture = UITapGestureRecognizer(target: self, action: #selector(handleWindowTap))
    gesture.cancelsTouchesInView = false
    window.addGestureRecognizer(gesture)
    windowTapGesture = gesture
  }

  private func removeWindowTapGesture() {
    guard let gesture = windowTapGesture else { return }
    gesture.view?.removeGestureRecognizer(gesture)
    windowTapGesture = nil
  }

  @objc private func handleWindowTap(_ gesture: UITapGestureRecognizer) {
    let location = gesture.location(in: self)
    if !bounds.contains(location) {
      blur()
    }
  }

  // MARK: - UITextField Delegate

  @objc func textFieldDidChange() {
    nativeEventCount += 1
    let text = textField.text ?? ""
    onTextChange(["text": text, "eventCount": nativeEventCount])
    onInputChange(["text": text, "eventCount": nativeEventCount])
  }

  func textFieldDidBeginEditing(_ textField: UITextField) {
    installWindowTapGesture()
    onInputFocus([:])
    if clearTextOnFocus {
      textField.text = ""
      onTextChange(["text": ""])
    } else if selectTextOnFocus {
      // Delay to ensure the field is ready
      DispatchQueue.main.async {
        textField.selectAll(nil)
      }
    }
  }

  func textFieldDidEndEditing(_ textField: UITextField) {
    removeWindowTapGesture()
    onInputBlur([:])
    onInputEndEditing(["text": textField.text ?? ""])
  }

  func textFieldDidChangeSelection(_ textField: UITextField) {
    guard let selectedRange = textField.selectedTextRange else { return }
    let start = textField.offset(from: textField.beginningOfDocument, to: selectedRange.start)
    let end = textField.offset(from: textField.beginningOfDocument, to: selectedRange.end)
    onInputSelectionChange(["selection": ["start": start, "end": end]])
  }

  func textFieldShouldReturn(_ textField: UITextField) -> Bool {
    let behavior = effectiveSubmitBehavior
    switch behavior {
    case "submit":
      onInputSubmitEditing(["text": textField.text ?? ""])
      return false
    case "newline":
      // Single-line cannot insert newline; fall back to blurAndSubmit
      onInputSubmitEditing(["text": textField.text ?? ""])
      textField.resignFirstResponder()
      return true
    default: // "blurAndSubmit"
      onInputSubmitEditing(["text": textField.text ?? ""])
      textField.resignFirstResponder()
      return true
    }
  }

  func textField(
    _ textField: UITextField,
    shouldChangeCharactersIn range: NSRange,
    replacementString string: String
  ) -> Bool {
    fireKeyPress(string)

    if textField.markedTextRange != nil { return true }

    guard let maxLength = maxLength else { return true }
    let currentText = textField.text ?? ""
    guard let swiftRange = Range(range, in: currentText) else { return true }
    let newText = currentText.replacingCharacters(in: swiftRange, with: string)
    return newText.count <= maxLength
  }

  func setShowDoneButton(_ show: Bool) {
    if show {
      if doneToolbar == nil {
        let toolbar = UIToolbar(frame: CGRect(x: 0, y: 0, width: UIScreen.main.bounds.width, height: 44))
        toolbar.sizeToFit()

        let flexSpace = UIBarButtonItem(barButtonSystemItem: .flexibleSpace, target: nil, action: nil)
        let doneButton = UIBarButtonItem(barButtonSystemItem: .done, target: self, action: #selector(handleDoneButtonTap))

        toolbar.items = [flexSpace, doneButton]
        doneToolbar = toolbar
      }

      textField.inputAccessoryView = doneToolbar
      textView.inputAccessoryView = doneToolbar
    } else {
      textField.inputAccessoryView = nil
      textView.inputAccessoryView = nil
    }
  }

  // MARK: - UITextView Delegate

  func textViewDidChange(_ textView: UITextView) {
    nativeEventCount += 1
    let text = textView.text ?? ""
    onTextChange(["text": text, "eventCount": nativeEventCount])
    onInputChange(["text": text, "eventCount": nativeEventCount])
    updatePlaceholderVisibility()
    notifyContentSizeChange()
  }

  func textViewDidBeginEditing(_ textView: UITextView) {
    installWindowTapGesture()
    onInputFocus([:])
    if clearTextOnFocus {
      textView.text = ""
      onTextChange(["text": ""])
      updatePlaceholderVisibility()
      notifyContentSizeChange()
    } else if selectTextOnFocus {
      DispatchQueue.main.async {
        textView.selectAll(nil)
      }
    }
  }

  func textViewDidEndEditing(_ textView: UITextView) {
    removeWindowTapGesture()
    onInputBlur([:])
    onInputEndEditing(["text": textView.text ?? ""])
  }

  func textViewDidChangeSelection(_ textView: UITextView) {
    let range = textView.selectedRange
    onInputSelectionChange([
      "selection": ["start": range.location, "end": range.location + range.length]
    ])
  }

  func textView(
    _ textView: UITextView,
    shouldChangeTextIn range: NSRange,
    replacementText text: String
  ) -> Bool {
    fireKeyPress(text)

    // Handle return key with submitBehavior
    if text == "\n" {
      let behavior = effectiveSubmitBehavior
      switch behavior {
      case "submit":
        onInputSubmitEditing(["text": textView.text ?? ""])
        return false
      case "blurAndSubmit":
        onInputSubmitEditing(["text": textView.text ?? ""])
        textView.resignFirstResponder()
        return false
      default: // "newline" — allow the newline to be inserted
        break
      }
    }

    if textView.markedTextRange != nil { return true }

    // maxLength check
    if let maxLength = maxLength {
      let currentText = textView.text ?? ""
      guard let swiftRange = Range(range, in: currentText) else { return true }
      let newText = currentText.replacingCharacters(in: swiftRange, with: text)
      if newText.count > maxLength { return false }
    }

    return true
  }

  // MARK: - Imperative Methods

  func focus() {
    if isMultiline {
      textView.becomeFirstResponder()
    } else {
      textField.becomeFirstResponder()
    }
  }

  func blur() {
    if isMultiline {
      textView.resignFirstResponder()
    } else {
      textField.resignFirstResponder()
    }
  }

  func clear() {
    if isMultiline {
      textView.text = ""
      onTextChange(["text": ""])
      updatePlaceholderVisibility()
      notifyContentSizeChange()
    } else {
      textField.text = ""
      onTextChange(["text": ""])
    }
  }

  func isFocused() -> Bool {
    isMultiline ? textView.isFirstResponder : textField.isFirstResponder
  }
}

