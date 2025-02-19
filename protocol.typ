// Concevoir le protocole expérimental qui quantifie a quel point les lignes a identifier sont bien indentifées
// (le protocole doit être de la collection des données jusqu'à l'analyse des données)

#set list(marker: [➢])

= Expérience

Le but est de savoir si les lignes pertinentes sont correctement identifiées.

== Collecte des données

100 paires de logs sont choisies aléatoirement du dataset en sélectionnant seulement les paires qui ont un failing log de 1000 lignes maximum.
(Pour avoir une annotation humaine plus qualitative)

_des écosystèmes sont sous-représenté dans l'expérience de fait qu'ils sont sous-représenté dans le dataset._

== Utilisation des données

1. Annotation de chaque log de failure par un humain afin de déterminer quelles lignes sont effectivement utiles.
	Une annotation correspond à la selection des lignes utiles.
	Dans le meilleur des cas il faudrait que 3 personnes annotent chaque failure log.
2. Executer l'algo cidiff sur chaque log pour produire une annotation du failure log.
3. Executer l'algo lcsdiff sur chaque log pour produire une annotation du failure log.
4. Comparer l'annotation de cidiff avec l'annotation humaine.
5. Comparer l'annotation de lcsdiff avec l'annotation humaine.

=== Annotation

L'annotation sélectionne les lignes pertinentes du log.

=== Annotation humaine

Un humain considère une ligne comme pertinente si elle apporte une quelconque information qui peut être utile pour analyser l'erreur.

_Il est possible d'affiner l'annotation en distinguant des critères sur la ligne (comme contexte/raison our indispensable/facultatif), mais il n'est pas encore nécessaire de le faire._

`Compiling package core: failure`: cette ligne apporte une information sur le contexte de l'erreur (erreur dans le package core) -> on la sélectionne\
`core/foo.java: syntax error`: cette ligne apporte une information sur la raison de l'erreur (erreur de syntaxe) -> on la sélectionne\
`Total time: 5.361s`: cette ligne n'apporte pas de contexte ni de raison de l'erreur -> on ne la sélectionne pas

=== Annotation cidiff/lcsdiff

Un algorithme considère une ligne comme pertinente si sont action de diff est "added". Les lignes qui n'ont pas cette action sont donc considérées non-pertinente.

== Analyse des données

L'analyse compare l'annotation de cidiff contre l'annotation humaine.
L'annotation humaine est considérée comme la vérité.
L'annotation cidiff/lcsdiff doit se raprocher le plus de l'annotation humaine.

- On calculera la précision et rappel de cidiff.

true positive: ligne sélectionnée par humain et cidiff\
true negative: ligne non sélectionnée par humain ni cidiff\
false positive: ligne sélectionnée par cidiff mais pas par humain (cidiff n'aurait pas du la sélectionner)\
false negative: ligne non sélectionnée par cidiff mais sélectionnée par humain (cidiff aurait du la sélectionner)

précision: proportions de lignes correctement sélectionnées parmis celles sélectionnées
$ "precision" = "true_positive" / ("true_positive" + "false_positive") $

rappel: proportiones de lignes correctement sélectionnées parmis toutes celles qui auraient du être sélectionnées
$ "rappel" = "true_positive" / ("true_positive" + "false_negative") $

- On pourra distinguer la précision et le rappel en fonction du langage du repository.

- On calculera aussi un graphe qui montre le nombre de lignes qui auraient du être sélectionnées mais ne l'ont pas été, ainsi que le nombre de lignes qui ont été sélectionnées mais qui n'auraient pas du l'être.

- On pourra aussi avoir un graphe de ces deux valeurs mais en proportions sur la quantité de lignes à sélectionner.

Il y a ainsi 100 valeurs de précisions et rappels. Cela permet aussi de faire une intervale de confiance.

