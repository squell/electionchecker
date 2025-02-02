Zetelverdelingsalgoritmevalidator
=================================

Deze code implementeert het zetelverdelingsalgoritme zoals dat is beschreven in de huidige Kieswet.

Meer specifiek, het algoritme zoals gebruikt wordt sinds 2017 voor:

* Tweede Kamerverkiezingen
* Provinciale Statenverkiezingen
* Gemeenteraadsverkiezingen
* Verkiezingen voor het Europees Parlement
* Waterschappen

Het is gevalideerd op alle verkiezingsdata die de [Kiesraad beschikbaar stelt](https://www.verkiezingsuitslagen.nl/).

Historische verkiezingen
------------------------

Historische Tweede Kamerverkiezingen zijn ook te valideren met deze code, m.u.v. de verkiezingen vanaf 1977 tot en met 2012. De
reden hiervoor is het systeem van "lijstcombinaties". Dit is nog niet geïmplementeerd (en ik zie daar ook weinig noodzaak
toe daar het in onbruik is geraakt), en het vereist ook handmatige correctie van de validatie data.

Dus behalve voor alle verkiezingen van na 2017 is het ook bruikbaar voor:

* De verkiezingen van 1937 tot en met 1972 (want deze gebruiken hetzelfde algoritme als dat van vandaag)

* De verkiezingen van 1925, 1929 en 1933 (een variatie op "grootste overschotten" in plaats van "grootste gemiddelden")

* De verkiezingen van 1918 en 1922 (met een rommelige restzetelverdeling). Hiervoor heb ik de testdata zelf aangepast op basis van bronnen zoals de Staatscourant, omdat de Kiesraad-data een onvolledige weergave van de situatie had)---hier valt op dat politieke partijen duidelijk het kiesstelsel hebben "gegamed": de KVP en ARP hebben samen hiermee drie extra zetels in de wacht gesleept dan waar ze anders recht op hadden gehad.
