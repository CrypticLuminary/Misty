from misty_ai.main import main


def test_main_smoke(capsys: object) -> None:
    # Import/startup smoke coverage; real queue behavior belongs to its milestone.
    main()
