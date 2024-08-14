import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart' show SvgPicture;

import 'package:mail_app/services/inbox_service.dart';
import 'package:mail_app/screens/home.dart';
import 'package:mail_app/types/project_colors.dart';
import 'package:mail_app/types/project_sizes.dart';

class SplashPage extends StatefulWidget {
  const SplashPage({super.key});

  @override
  SplashPageState createState() => SplashPageState();
}

class SplashPageState extends State<SplashPage> {
  double _turns = 0;
  String _status = '';
  bool _loadingFinished = false;

  @override
  void initState() {
    super.initState();

    _loadHomePage();
  }

  void _loadHomePage() async {
    _refreshRotate();

    setState(() => _status = 'Loading inboxes');
    final inboxService = await _loadInboxService();

    _loadingFinished = true;

    if (!mounted) return;
    Navigator.pushReplacement(
      context,
      MaterialPageRoute(
        builder: (context) => HomePage(
          inboxService: inboxService,
        ),
      ),
    );
  }

  Future<InboxService> _loadInboxService() async {
    final inboxService = InboxService();

    final sessions = await inboxService.getSessions();

    if (sessions == null) {
      final inboxService = await _loadInboxService();

      return inboxService;
    }

    if (sessions.isNotEmpty) {
      inboxService.setActiveSessionId(sessions[0].sessionId);
      inboxService.updateMailboxes();
    }

    return inboxService;
  }

  void _refreshRotate() async {
    setState(() {
      _turns += 1;
    });

    await Future.delayed(const Duration(seconds: 1), () {});

    if (!_loadingFinished) _refreshRotate();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Container(
        decoration: BoxDecoration(
          color: ProjectColors.background(true),
        ),
        child: Center(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              Padding(
                padding: const EdgeInsets.only(bottom: 30),
                child: Text(
                  _status,
                  style: TextStyle(
                    fontSize: ProjectSizes.fontSizeExtraLarge,
                    color: ProjectColors.text(true),
                  ),
                ),
              ),
              AnimatedRotation(
                alignment: Alignment.center,
                turns: _turns,
                duration: const Duration(seconds: 1),
                child: SvgPicture.asset(
                  'assets/icons/arrows-rotate.svg',
                  color: ProjectColors.text(true),
                  width: 60,
                  height: 60,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
