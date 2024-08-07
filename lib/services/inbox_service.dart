import 'package:mail_app/services/http_service.dart';
import 'package:mail_app/types/http_request_path.dart';
import 'package:mail_app/types/mail_account.dart';
import 'package:mail_app/types/mailbox_info.dart';
import 'package:mail_app/types/message.dart';
import 'package:mail_app/types/message_flag.dart';

class InboxService {
  int? _activeSession;
  String? _activeMailbox;

  final HttpService httpService = HttpService();

  final List<MailAccount> _sessions = [];
  final List<MailboxInfo> _mailboxes = [];

  void setActiveSessionId(int session) {
    _activeSession = session;
  }

  int? getActiveSessionId() {
    return _activeSession;
  }

  void setActiveMailbox(String mailbox) {
    _activeMailbox = mailbox;
  }

  String? getActiveSessionDisplay() {
    final sessions =
        _sessions.where((element) => element.sessionId == _activeSession);

    if (sessions.isEmpty) return '';

    return sessions.first.username;
  }

  String? getActiveMailbox() {
    return _activeMailbox;
  }

  String? getActiveMailboxDisplay() {
    final mailboxes =
        _mailboxes.where((element) => element.path == _activeMailbox);

    if (mailboxes.isEmpty) return '';

    return mailboxes.first.display;
  }

  Future<Map<int, List<MailboxInfo>>> getMailboxTree() async {
    Map<int, List<MailboxInfo>> mailboxTree = {};

    for (var session in _sessions) {
      final mailboxes = await getMailboxes(session: session.sessionId);

      mailboxTree[session.sessionId] = mailboxes;
    }

    return mailboxTree;
  }

  Future<int> newSession(
    String username,
    String password,
    String address,
    int port,
  ) async {
    final body = {
      'email': username,
      'password': password,
      'address': address,
      'port': port.toString(),
    };

    final messageData =
        await HttpService().sendRequest(HttpRequestPath.login, body);

    if (!messageData.success) return -1;

    final session = messageData.data['id'] as int;

    _sessions.add(MailAccount(
      session,
      username,
      address,
      port,
    ));

    return session;
  }

  Future<List<MailAccount>> getSessions() async {
    final messageData =
        await httpService.sendRequest(HttpRequestPath.get_sessions, {});

    if (!messageData.success) return [];

    for (var session in (messageData.data as List)) {
      _sessions.add(MailAccount.fromJson(session));
    }

    return _sessions;
  }

  Future<List<MailboxInfo>> getMailboxes({int? session}) async {
    if (session == null) {
      session = _activeSession;

      if (_activeSession == null) return [];
    }

    final body = {
      'session_id': session.toString(),
    };

    final messageData =
        await HttpService().sendRequest(HttpRequestPath.get_mailboxes, body);

    if (!messageData.success) return [];

    _mailboxes.clear();

    for (var mailbox in (messageData.data as List)) {
      if ((mailbox as String).endsWith(']')) continue;
      _mailboxes
          .add(MailboxInfo.fromJson(mailbox, getActiveSessionDisplay() ?? ''));
    }

    return _mailboxes;
  }

  Future<List<Message>> getMessages({
    int? session,
    String? mailbox,
    required int start,
    required int end,
  }) async {
    if (session == null) {
      session = _activeSession;

      if (_activeSession == null) return [];
    }
    if (mailbox == null) {
      mailbox = _activeMailbox;

      if (_activeMailbox == null) return [];
    }

    final body = {
      'session_id': session.toString(),
      'mailbox_path': mailbox!,
      'start': start.toString(),
      'end': end.toString(),
    };

    final messageData =
        await HttpService().sendRequest(HttpRequestPath.get_messages, body);

    if (!messageData.success) return [];

    final List<Message> messages = [];
    for (var message in (messageData.data as List)) {
      messages.add(Message.fromJson(message));
    }

    return messages;
  }

  Future<List<MessageFlag>> modifyFlags({
    int? session,
    String? mailbox,
    required int messageUid,
    required List<MessageFlag> flags,
    required bool add,
  }) async {
    if (session == null) {
      session = _activeSession;

      if (_activeSession == null) return [];
    }
    if (mailbox == null) {
      mailbox = _activeMailbox;

      if (_activeMailbox == null) return [];
    }

    final flagsString = flags.map((e) => e.name).join(',');
    final addString = add.toString();

    final body = {
      'session_id': session.toString(),
      'mailbox_path': mailbox!,
      'message_uid': messageUid.toString(),
      'flags': flagsString,
      'add': addString,
    };

    final messageData =
        await HttpService().sendRequest(HttpRequestPath.modify_flags, body);

    if (!messageData.success) return [];

    return messageFlagsFromJsonList(messageData.data);
  }

  Future<String> moveMessage({
    int? session,
    String? mailbox,
    required int messageUid,
    required String mailboxDest,
  }) async {
    if (session == null) {
      session = _activeSession;

      if (_activeSession == null) return '';
    }
    if (mailbox == null) {
      mailbox = _activeMailbox;

      if (_activeMailbox == null) return '';
    }

    final body = {
      'session_id': session.toString(),
      'mailbox_path': mailbox!,
      'message_uid': messageUid.toString(),
      'mailbox_path_dest': mailboxDest,
    };

    final messageData =
        await HttpService().sendRequest(HttpRequestPath.move_message, body);

    if (!messageData.success) return '';

    return messageData.data;
  }
}
