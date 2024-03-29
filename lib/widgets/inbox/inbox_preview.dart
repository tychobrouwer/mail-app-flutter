import 'package:flutter/material.dart';
import 'package:intl/intl.dart';

import 'package:mail_app/types/project_colors.dart';
import 'package:mail_app/mail-client/enough_mail.dart';

class MailPreview extends StatefulWidget {
  final MimeMessage email;
  final int idx;
  final bool unseen;
  final Function getActive;
  final Function updateMessageID;

  const MailPreview({
    super.key,
    required this.email,
    required this.idx,
    required this.unseen,
    required this.getActive,
    required this.updateMessageID,
  });

  @override
  MailPreviewState createState() => MailPreviewState();
}

class MailPreviewState extends State<MailPreview> {
  late MimeMessage _email;
  late int _idx;
  late bool _unseen;
  late Function _getActive;
  late Function _updateMessageID;

  late String _from;
  late String _previewStr;
  late String _dateText;

  @override
  void initState() {
    super.initState();

    _email = widget.email;
    _idx = widget.idx;
    _unseen = widget.unseen;
    _getActive = widget.getActive;
    _updateMessageID = widget.updateMessageID;

    DateTime? date = _email.decodeDate();

    _dateText = date == null
        ? "Unknown date"
        : DateTime.now().difference(date).inDays == 0
            ? DateFormat('HH:mm').format(date)
            : DateTime.now().difference(date).inDays == -1
                ? 'Yesterday'
                : DateFormat('dd/MM/yy').format(date);

    _from = _email.from![0].personalName ?? _email.from![0].email;

    _previewStr = _textPreview();
  }

  String _textPreview() {
    try {
      if (_email.decodeTextHtmlPart() != null) {
        return _htmlPreview() ?? _plainTextPreview();
      } else {
        return _plainTextPreview();
      }
    } catch (e) {
      return _plainTextPreview();
    }
  }

  String? _htmlPreview() {
    final html = _email.decodeTextHtmlPart()!;
    final decoded = html
        .replaceAll(RegExp(r"\n|\r"), "")
        .replaceAll(RegExp(r"( +|&nbsp;)"), " ")
        .replaceAll(RegExp(r"&amp;"), "&");
    final previewRegex =
        RegExp(r'(?<=>)([^\/<>}\n]{5,})(?=<)').firstMatch(decoded);

    if (previewRegex == null) {
      return null;
    } else {
      return previewRegex[0]!.trim();
    }
  }

  String _plainTextPreview() {
    if (_email.decodeTextPlainPart() != null) {
      return _email.decodeTextPlainPart()!.split(RegExp(r"\n"))[0];
    } else {
      return '';
    }
  }

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: () => _updateMessageID(_idx),
      child: Container(
          decoration: BoxDecoration(
            borderRadius: const BorderRadius.all(
              Radius.circular(10),
            ),
            color: _getActive(_idx) ? ProjectColors.accent : Colors.transparent,
          ),
          child: Container(
            margin: const EdgeInsets.only(left: 10, right: 30),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Container(
                  margin: const EdgeInsets.only(top: 13, right: 10),
                  width: 10,
                  height: 10,
                  decoration: BoxDecoration(
                      borderRadius: BorderRadius.circular(5),
                      color: _unseen
                          ? !_getActive(_idx)
                              ? ProjectColors.accent
                              : ProjectColors.main(true)
                          : Colors.transparent),
                ),
                Expanded(
                  child: Container(
                    decoration: BoxDecoration(
                      border: Border(
                        bottom: BorderSide(
                          color: !_getActive(_idx)
                              ? ProjectColors.secondary(_getActive(_idx))
                              : Colors.transparent,
                        ),
                      ),
                    ),
                    child: Padding(
                      padding: const EdgeInsets.only(bottom: 10, top: 8),
                      child: Column(
                        children: [
                          Align(
                            alignment: Alignment.centerLeft,
                            child: Row(
                              children: [
                                Expanded(
                                  child: Padding(
                                    padding: const EdgeInsets.only(right: 10),
                                    child: Text(
                                      _from,
                                      overflow: TextOverflow.fade,
                                      softWrap: false,
                                      style: TextStyle(
                                        fontSize: 14,
                                        fontWeight: FontWeight.bold,
                                        color: ProjectColors.main(
                                            _getActive(_idx)),
                                      ),
                                    ),
                                  ),
                                ),
                                Text(
                                  _dateText,
                                  style: TextStyle(
                                    color: ProjectColors.secondary(
                                        _getActive(_idx)),
                                    fontSize: 12,
                                  ),
                                ),
                              ],
                            ),
                          ),
                          Align(
                            alignment: Alignment.centerLeft,
                            child: Text(
                              _email.decodeSubject() ?? '',
                              overflow: TextOverflow.fade,
                              softWrap: false,
                              style: TextStyle(
                                fontSize: 13,
                                color: ProjectColors.main(_getActive(_idx)),
                                fontWeight: FontWeight.w500,
                              ),
                            ),
                          ),
                          Align(
                            alignment: Alignment.centerLeft,
                            child: Text(
                              _previewStr,
                              overflow: TextOverflow.fade,
                              softWrap: false,
                              style: TextStyle(
                                fontSize: 13,
                                color:
                                    ProjectColors.secondary(_getActive(_idx)),
                                fontWeight: FontWeight.w500,
                              ),
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
                ),
              ],
            ),
          )),
    );
  }
}
